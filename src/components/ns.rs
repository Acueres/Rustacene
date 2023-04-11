use super::*;
use bevy::prelude::Component;
use bevy::utils::HashMap;
use petgraph::graph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::Direction;
use rand::prelude::SliceRandom;
use std::collections::{HashSet, VecDeque};

pub const NS_ENERGY_COST: f32 = 1e-6;

#[derive(Copy, Clone)]
struct NodeConnection {
    pub incoming: usize,
    pub outgoing: usize,
    pub weight: f32,
}

impl NodeConnection {
    pub fn new(incoming: usize, outgoing: usize, weight: f32) -> Self {
        Self {
            incoming,
            outgoing,
            weight,
        }
    }
}

#[derive(Component, Clone)]

pub struct NeuralSystem {
    ns_shape: NsShape,
    nodes: Vec<NodeConnection>,
    weights: HashMap<usize, f32>,
    sources: HashSet<usize>,
}

impl NeuralSystem {
    pub fn new(connections: &Vec<Connection>, ns_shape: NsShape) -> Self {
        let mut nn_graph = Graph::<f32, f32>::with_capacity(ns_shape.n_neurons, connections.len());

        for _ in 0..ns_shape.n_neurons {
            nn_graph.add_node(0.);
        }

        for c in connections.iter() {
            nn_graph.add_edge(NodeIndex::new(c.in_index), NodeIndex::new(c.out_index), c.w);
        }

        let out_start = ns_shape.input + ns_shape.hidden;
        let mut nodes = Vec::<NodeConnection>::new();
        let mut nodes_queue = VecDeque::<usize>::from_iter(out_start..ns_shape.n_neurons);
        let mut connected = HashSet::<usize>::from_iter(0..ns_shape.input);
        connected.extend(out_start..ns_shape.n_neurons);

        while nodes_queue.len() > 0 {
            let index = nodes_queue.pop_front().unwrap();
            let neighbors = nn_graph.neighbors_directed(NodeIndex::new(index), Direction::Incoming);
            let mut walk = neighbors.detach();
            let n_incoming = neighbors.count();

            if n_incoming == 0 {
                continue;
            }

            let mut incoming = Vec::<NodeConnection>::with_capacity(n_incoming);

            while let Some((edge, node)) = walk.next(&nn_graph) {
                let weight = *nn_graph.edge_weight(edge).unwrap();
                incoming.push(NodeConnection::new(node.index(), index, weight));
                if connected.insert(node.index()) {
                    nodes_queue.push_back(node.index());
                }
            }
            nodes = [nodes, incoming].concat();
        }

        nodes.reverse();

        let mut sources = HashSet::<usize>::from_iter(0..ns_shape.input);
        let output = HashSet::<usize>::from_iter(out_start..ns_shape.n_neurons);
        let hidden = &(&connected - &sources) - &output;

        for node in connected.iter() {
            let neighbors = nn_graph.neighbors_directed(NodeIndex::new(*node), Direction::Incoming);
            let mut walk = neighbors.detach();
            let n_incoming = neighbors.count();

            if !hidden.contains(&node) {
                continue;
            }
            if n_incoming == 0 {
                sources.insert(*node);
            } else {
                let mut self_connected = true;
                while let Some((_, node_index)) = walk.next(&nn_graph) {
                    if node_index.index() != *node {
                        self_connected = false;
                        break;
                    }
                }

                if self_connected {
                    sources.insert(*node);
                }
            }
        }

        let mut weights =
            HashMap::<usize, f32>::from_iter(connected.iter().map(|node| (*node, 0.)));

        // init sources
        for source_index in sources.iter() {
            weights.entry(*source_index).and_modify(|f| *f = 0.5);
        }

        Self {
            nodes,
            ns_shape,
            weights,
            sources,
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
        //set inputs
        for input_index in 0..self.ns_shape.input {
            self.weights
                .entry(input_index)
                .and_modify(|f| *f = input[input_index]);
        }
        let mut activated_nodes = HashMap::<usize, bool>::from_iter(
            self.weights
                .iter()
                .map(|(k, _)| (*k, (0..self.ns_shape.input).contains(k))),
        );

        let previous_weights = self.weights.clone();

        for conn in self.nodes.iter() {
            let self_connected = conn.incoming == conn.outgoing;
            let activated = *activated_nodes.get(&conn.incoming).unwrap() || self_connected;
            let mut weight = if self_connected {
                *previous_weights.get(&conn.incoming).unwrap()
            } else {
                *self.weights.get(&conn.incoming).unwrap()
            };
            if !activated {
                weight = weight.tanh();

                self.weights
                    .entry(conn.incoming)
                    .and_modify(|f| *f = weight);

                activated_nodes
                    .entry(conn.incoming)
                    .and_modify(|v| *v = true);
            }

            self.weights
                .entry(conn.outgoing)
                .and_modify(|f| *f += weight * conn.weight);
        }

        let out_start = self.ns_shape.input + self.ns_shape.hidden;
        let out_end = out_start + self.ns_shape.output;

        let mut res = vec![0.0f32; self.ns_shape.output];
        for node in out_start..out_end {
            let weight = *self.weights.get(&node).unwrap();
            res[node - out_start] = weight;
        }

        let nodes_to_clear = &HashSet::<usize>::from_iter(self.weights.keys().map(|node| *node))
            - &HashSet::<usize>::from_iter(self.sources.iter().cloned());

        for node in nodes_to_clear {
            self.weights.entry(node).and_modify(|f| *f = 0.);
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

        assert_eq!(ns.weights.iter().len(), 5);
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
    fn test_output_concurrency() {
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
        let weight_node_1: f32 = (0.5_f32.tanh() as f32 * 0.4).tanh();
        let weight_node_0: f32 = (weight_node_1 * 0.6 + input[0]).tanh();
        let expected_output = *softmax(&vec![weight_node_0 * 0.5]).first().unwrap();

        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e9) as usize,
            (expected_output * 1e9) as usize
        );
    }

    #[test]
    fn test_self_connected() {
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
    fn test_pruning_complex() {
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
        assert_eq!(ns_shape.n_neurons - 1, ns.weights.len());
        assert_eq!(connections.len() - 1, ns.nodes.len());
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
