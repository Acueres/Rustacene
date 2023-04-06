use super::*;
use bevy::prelude::Component;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{Dfs, Reversed};
use petgraph::Direction;
use rand::prelude::SliceRandom;
use std::collections::{HashSet, VecDeque};

pub const NS_ENERGY_COST: f32 = 1e-6;

#[derive(Component, Clone)]

pub struct NeuralSystem {
    ns_shape: NsShape,
    nn_graph: StableGraph<f32, f32>,
    sources: Vec<usize>,
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

        let reversed = Reversed(&nn_graph);

        let mut connected_nodes = HashSet::<usize>::new();
        let out_start = ns_shape.input + ns_shape.hidden;

        for index in out_start..ns_shape.n_neurons {
            let node = NodeIndex::new(index);

            let mut dfs = Dfs::new(&reversed, node);

            while let Some(visited) = dfs.next(&reversed) {
                connected_nodes.insert(visited.index());
            }
        }

        let mut sources = HashSet::<usize>::from_iter(0..ns_shape.input);
        let output = HashSet::<usize>::from_iter(out_start..ns_shape.n_neurons);
        let hidden = &(&connected_nodes - &sources) - &output;

        let unconnected_nodes: HashSet<usize> =
            &HashSet::from_iter((0..ns_shape.n_neurons).into_iter()) - &connected_nodes;
        let unconnected_nodes = &unconnected_nodes - &sources;

        for index in unconnected_nodes {
            nn_graph.remove_node(NodeIndex::new(index));
        }

        for index in hidden {
            let neighbors = nn_graph.neighbors_directed(NodeIndex::new(index), Direction::Incoming);
            let mut walk = neighbors.detach();
            let n_incoming = neighbors.count();

            if n_incoming == 0 {
                sources.insert(index);
            } else if n_incoming == 1 {
                // self-connected
                if let Some((_, node)) = walk.next(&nn_graph) {
                    if node.index() == index {
                        sources.insert(index);
                    }
                }
            }
        }

        let mut sources = Vec::<usize>::from_iter(sources);
        sources.sort();

        // init sources
        for source_index in sources.iter() {
            *nn_graph
                .node_weight_mut(NodeIndex::new(*source_index))
                .unwrap() = 0.5;
        }

        Self {
            nn_graph,
            ns_shape,
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
        let mut res = vec![0.0f32; self.ns_shape.output];

        //set inputs
        for input_index in 0..self.ns_shape.input {
            *self
                .nn_graph
                .node_weight_mut(NodeIndex::new(input_index))
                .unwrap() = input[input_index];
        }

        let out_start = self.ns_shape.input + self.ns_shape.hidden;
        let out_end = out_start + self.ns_shape.output;

        let mut visited = HashSet::<usize>::new();
        let mut nodes = VecDeque::<usize>::from_iter(self.sources.iter().cloned());

        while nodes.len() > 0 {
            let node_index = nodes.pop_front().unwrap();
            let mut node_out = *self
                .nn_graph
                .node_weight(NodeIndex::new(node_index))
                .unwrap();

            if (out_start..out_end).contains(&node_index) {
                res[node_index - out_start] = sigmoid(node_out);
                continue;
            }

            if !self.sources.contains(&node_index) {
                *self
                    .nn_graph
                    .node_weight_mut(NodeIndex::new(node_index))
                    .unwrap() = 0.;
                node_out = node_out.tanh();
            }

            let mut neighbors = self
                .nn_graph
                .neighbors_directed(NodeIndex::new(node_index), Direction::Outgoing)
                .detach();

            while let Some((edge, node)) = neighbors.next(&self.nn_graph) {
                let edge_weight = *self.nn_graph.edge_weight(edge).unwrap();
                *self.nn_graph.node_weight_mut(node).unwrap() += node_out * edge_weight;
                if visited.insert(node.index()) {
                    nodes.push_back(node.index());
                }
            }
        }

        res
    }
}

#[inline]
fn sigmoid(value: f32) -> f32 {
    const EPS: f32 = 1e-8;
    let exp = 1. / value.exp();
    1. / (1. + exp + EPS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn test_ns_output_and_pruning() {
        let ns_shape = NsShape::new(3, 2, 1);

        let connections = vec![
            renumber_conn_indexes(&Connection::new(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(1., true, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, false, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::new(0.3, true, false, 2, 1), &ns_shape),
        ];

        let mut ns = NeuralSystem::new(&connections, ns_shape);

        assert_eq!(ns.nn_graph.node_count(), 5);
        assert_eq!(ns.sources, vec![0, 1, 2]);

        let test_input = vec![0.5, 0.8];
        let out_inner = test_input.iter().sum::<f32>().tanh();
        let test_output: f32 = sigmoid(out_inner * 0.3);

        let input = vec![0.5, 0.8, 1.]; // last value should not affect the output
        let output = *ns.forward(&input).first().unwrap();

        assert_eq!((output * 1e9) as usize, (test_output * 1e9) as usize);
    }

    #[test]
    fn test_ns_pruning_complex() {
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
            vec![0, 1, 2, 3, ns_shape.input + 1, ns_shape.input + 2]
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
