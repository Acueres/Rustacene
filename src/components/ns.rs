use super::*;
use bevy::prelude::Component;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::DfsPostOrder;
use petgraph::visit::Reversed;
use petgraph::Direction;
use rand::prelude::SliceRandom;
use std::collections::{HashMap, HashSet};

pub const NS_ENERGY_COST: f32 = 1e-6;

#[derive(Component, Clone)]

pub struct NeuralSystem {
    ns_shape: NsShape,
    nn_graph: StableGraph<f32, f32>,
    nodes: Vec<usize>,
    sources: HashSet<usize>,
    self_connected: HashSet<usize>,
}

impl NeuralSystem {
    pub fn new(connections: &Vec<Connection>, ns_shape: NsShape) -> Self {
        let mut nn_graph =
            StableGraph::<f32, f32>::with_capacity(ns_shape.n_neurons, connections.len());

        for _ in 0..ns_shape.n_neurons {
            nn_graph.add_node(0.);
        }

        for c in connections.iter() {
            nn_graph.add_edge(NodeIndex::new(c.in_index), NodeIndex::new(c.out_index), c.w);
        }

        let reversed_graph = Reversed(&nn_graph);
        let out_start = ns_shape.input + ns_shape.hidden;
        let mut visited = HashSet::<usize>::new();
        let mut nodes = Vec::<usize>::new();

        for index in out_start..ns_shape.n_neurons {
            let index = NodeIndex::new(index);

            let mut dfs = DfsPostOrder::new(&reversed_graph, index);

            while let Some(node) = dfs.next(&reversed_graph) {
                if visited.insert(node.index()) {
                    nodes.push(node.index());
                }
            }
        }

        let nodes_to_remove = &HashSet::<usize>::from_iter(0..ns_shape.n_neurons)
            - &HashSet::<usize>::from_iter(nodes.iter().cloned());
        let nodes_to_remove = &nodes_to_remove - &HashSet::<usize>::from_iter(0..ns_shape.input);

        for index in nodes_to_remove.into_iter() {
            nn_graph.remove_node(NodeIndex::new(index));
        }

        let mut sources = HashSet::<usize>::from_iter(0..ns_shape.input);
        let mut self_connected = HashSet::<usize>::new();

        for index in nodes.iter() {
            let neighbors =
                nn_graph.neighbors_directed(NodeIndex::new(*index), Direction::Incoming);
            let mut walk = neighbors.detach();
            let n_incoming = neighbors.count();

            if n_incoming == 0 {
                sources.insert(*index);
                continue;
            }

            let mut incoming = Vec::<usize>::with_capacity(n_incoming);
            while let Some(node) = walk.next_node(&nn_graph) {
                incoming.push(node.index());
            }

            let n_self = incoming
                .iter()
                .fold(0, |acc, n| if n == index { acc + 1 } else { acc });

            if n_self > 0 {
                self_connected.insert(*index);
            }

            if n_self == incoming.len() {
                sources.insert(*index);
            }
        }

        // init sources
        for source_index in sources.iter() {
            *nn_graph
                .node_weight_mut(NodeIndex::new(*source_index))
                .unwrap() = 0.5;
        }

        Self {
            ns_shape,
            nn_graph,
            nodes,
            sources,
            self_connected,
        }
    }

    pub fn get_action(&mut self, input: Vec<f32>) -> Action {
        let mut rng = rand::thread_rng();

        let probas: Vec<_> = self
            .forward(&input)
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.to_owned()))
            .collect();

        let action_index = probas
            .choose_weighted(&mut rng, |(_, p)| *p)
            .unwrap_or(&(0, 0.))
            .0;

        Action::get(action_index)
    }

    pub fn forward(&mut self, input: &Vec<f32>) -> Vec<f32> {
        //set sources
        for index in self.sources.iter() {
            *self
                .nn_graph
                .node_weight_mut(NodeIndex::new(*index))
                .unwrap() = if (0..self.ns_shape.input).contains(index) {
                input[*index] //inputs
            } else {
                0.5 //internal sources
            };
        }

        let mut self_connected_weights =
            HashMap::<usize, f32>::with_capacity(self.self_connected.len());
        for index in self.self_connected.iter() {
            let w = *self.nn_graph.node_weight(NodeIndex::new(*index)).unwrap();
            self_connected_weights.insert(*index, w);
        }

        let nodes_to_clear =
            &HashSet::<usize>::from_iter(self.nodes.iter().cloned()) - &self.sources;
        for index in nodes_to_clear.into_iter() {
            *self
                .nn_graph
                .node_weight_mut(NodeIndex::new(index))
                .unwrap() = 0.;
        }

        let hidden_range = self.ns_shape.input..self.ns_shape.input + self.ns_shape.hidden;

        for index in self.nodes.iter() {
            let node = NodeIndex::new(*index);

            let neighbors = self.nn_graph.neighbors_directed(node, Direction::Outgoing);

            if self.self_connected.contains(index) {
                let mut walk = neighbors.detach();
                let n_outgoing = neighbors.count();
                let mut outgoing = Vec::<(NodeIndex, f32)>::with_capacity(n_outgoing);
                let self_connected_weight = *self_connected_weights.get(index).unwrap();

                while let Some((edge, next_node)) = walk.next(&self.nn_graph) {
                    let edge_weight = *self.nn_graph.edge_weight(edge).unwrap();
                    outgoing.push((next_node, edge_weight));
                }

                let (self_out, other_conns): (Vec<(NodeIndex, f32)>, Vec<(NodeIndex, f32)>) =
                    outgoing.into_iter().partition(|x| x.0.index() == *index);

                //propagate to self
                for conn in self_out.into_iter() {
                    *self.nn_graph.node_weight_mut(node).unwrap() += conn.1 * self_connected_weight;
                }

                //activate self-connected
                let activated_node_weight = (*self.nn_graph.node_weight(node).unwrap()).tanh();
                *self.nn_graph.node_weight_mut(node).unwrap() = activated_node_weight;

                //propagate to others
                for conn in other_conns.into_iter() {
                    *self.nn_graph.node_weight_mut(conn.0).unwrap() +=
                        conn.1 * activated_node_weight;
                }
            } else {
                let mut walk = neighbors.detach();

                let mut node_weight = *self.nn_graph.node_weight(node).unwrap();

                if hidden_range.contains(index)
                    && !self.self_connected.contains(index)
                    && !self.sources.contains(index)
                {
                    node_weight = node_weight.tanh();
                    *self.nn_graph.node_weight_mut(node).unwrap() = node_weight;
                }

                while let Some((edge, next_node)) = walk.next(&self.nn_graph) {
                    let edge_weight = *self.nn_graph.edge_weight(edge).unwrap();
                    *self.nn_graph.node_weight_mut(next_node).unwrap() += edge_weight * node_weight;
                }
            }
        }

        let out_start = self.ns_shape.input + self.ns_shape.hidden;
        let out_end = out_start + self.ns_shape.output;

        //get outputs
        let mut res = vec![0.; self.ns_shape.output];
        for index in out_start..out_end {
            let node = NodeIndex::new(index);
            let weight = *self.nn_graph.node_weight(node).unwrap();
            res[index - out_start] = weight;
        }

        softmax(&res)
    }
}

#[inline]
fn softmax(input: &Vec<f32>) -> Vec<f32> {
    let exp: Vec<f32> = input.iter().map(|v| v.exp()).collect();
    let sum: f32 = exp.iter().sum();
    input.iter().map(|v| v / sum).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn test_output_and_pruning() {
        let ns_shape = NsShape::new(3, 2, 1);

        let connections = vec![
            renumber_conn_indexes(&Connection::new(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., true, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, false, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, true, false, 2, 1), &ns_shape),
        ];

        let mut ns = NeuralSystem::new(&connections, ns_shape);

        assert_eq!(ns.nn_graph.node_count(), 5);
        assert_eq!(ns.sources, HashSet::<usize>::from_iter(vec![0, 1, 2]));

        let input = vec![0.5, 0.8];
        let out_inner = input.iter().sum::<f32>().tanh();
        let expected_output = *softmax(&vec![out_inner * 0.3]).first().unwrap();

        let input = vec![0.5, 0.8, 1.]; // last value should not affect the output
        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e9) as usize,
            (expected_output * 1e9) as usize
        );
    }

    #[test]
    fn test_output_node_ordering() {
        let ns_shape = NsShape::new(1, 3, 1);

        let mut connections = vec![
            renumber_conn_indexes(&Connection::new(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.6, false, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.4, false, false, 2, 1), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.5, false, true, 0, 0), &ns_shape),
        ];
        connections.shuffle(&mut rand::thread_rng());

        let mut ns = NeuralSystem::new(&connections, ns_shape);
        assert_eq!(2, ns.sources.len());

        let input = vec![0.8];
        let weight_node_1: f32 = (0.5 as f32 * 0.4).tanh();
        let weight_node_0: f32 = (weight_node_1 * 0.6 + input[0]).tanh();
        let expected_output = *softmax(&vec![weight_node_0 * 0.5]).first().unwrap();

        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e9) as usize,
            (expected_output * 1e9) as usize
        );
    }

    #[test]
    fn test_self_connected_source() {
        let ns_shape = NsShape::new(1, 1, 1);

        let connections = vec![
            renumber_conn_indexes(&Connection::new(0.7, false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.2, false, true, 0, 0), &ns_shape),
        ];

        let mut ns = NeuralSystem::new(&connections, ns_shape);
        assert_eq!(2, ns.sources.len());

        let mut weight: f32 = 0.5;
        weight += 0.5 * 0.7;
        weight += 0.5 * 1.;
        weight += 0.5 * 0.3;
        weight = weight.tanh();
        let expected_output = *softmax(&vec![weight * 0.2]).first().unwrap();

        let input = vec![0.];
        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e9) as usize,
            (expected_output * 1e9) as usize
        );
    }

    #[test]
    fn test_self_connected() {
        let ns_shape = NsShape::new(2, 1, 1);

        let connections = vec![
            renumber_conn_indexes(&Connection::new(1.2, true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.9, true, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.7, false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, false, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.2, false, true, 0, 0), &ns_shape),
        ];

        let mut ns = NeuralSystem::new(&connections, ns_shape);
        //set a weight to self-connected node
        *ns.nn_graph.node_weight_mut(NodeIndex::new(2)).unwrap() = 0.74;

        let input = vec![0.9, 0.4];
        let mut weight: f32 = 0.;
        //self-connected
        weight += 0.74 * 0.7;
        weight += 0.74 * 1.;
        weight += 0.74 * 0.3;
        //other
        weight += input[0] * 1.2;
        weight += input[1] * 0.9;
        weight = weight.tanh();
        let expected_output = *softmax(&vec![weight * 0.2]).first().unwrap();

        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e6) as usize,
            (expected_output * 1e6) as usize
        );
    }

    #[test]
    fn test_sources_and_pruning() {
        let ns_shape = NsShape::new(4, 4, 2);

        let mut connections = vec![
            //input to internal
            renumber_conn_indexes(&Connection::new(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., true, false, 1, 0), &ns_shape),
            //input to output
            renumber_conn_indexes(&Connection::new(1., true, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., true, true, 2, 0), &ns_shape),
            //self-connected
            renumber_conn_indexes(&Connection::new(1., false, false, 1, 1), &ns_shape),
            //internal to internal
            renumber_conn_indexes(&Connection::new(1., false, false, 1, 0), &ns_shape),
            //internal to output
            renumber_conn_indexes(&Connection::new(1., false, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., false, true, 2, 1), &ns_shape),
            //input to internal unconnected
            renumber_conn_indexes(&Connection::new(1., true, false, 3, 3), &ns_shape),
        ];
        //ensure connections ordering doesn't matter
        connections.shuffle(&mut rand::thread_rng());

        let ns = NeuralSystem::new(&connections, ns_shape);

        //test sources
        assert_eq!(
            ns.sources,
            HashSet::<usize>::from_iter(vec![0, 1, 2, 3, ns_shape.input + 1, ns_shape.input + 2])
        );

        //test pruning
        assert_eq!(ns_shape.n_neurons - 1, ns.nn_graph.node_count());
        assert_eq!(connections.len() - 1, ns.nn_graph.edge_count());
    }

    #[inline]
    fn renumber_conn_indexes(conn: &Connection, ns_shape: &NsShape) -> Connection {
        let in_index = renumber_in_index(conn.in_index, conn.sensor_in, ns_shape.input);
        let out_index = renumber_out_index(
            conn.out_index,
            conn.sensor_out,
            ns_shape.input,
            ns_shape.input + ns_shape.hidden,
        );

        Connection::new(conn.w, conn.sensor_in, conn.sensor_out, in_index, out_index)
    }

    #[inline]
    fn renumber_in_index(index: usize, condition: bool, offset: usize) -> usize {
        if condition {
            return index;
        }
        index + offset
    }

    #[inline]
    fn renumber_out_index(index: usize, condition: bool, offset1: usize, offset2: usize) -> usize {
        if condition {
            return index + offset2;
        }
        index + offset1
    }
}
