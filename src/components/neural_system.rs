use super::*;
use bevy::prelude::Component;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{DfsPostOrder, Reversed};
use petgraph::Direction;
use rand::prelude::SliceRandom;
use std::collections::{HashMap, HashSet};

#[derive(Component, Clone)]
pub struct NeuralSystem {
    ns_shape: NsShape,
    nn_graph: StableGraph<Neuron, f32>,
    nodes: Vec<usize>,
    sources: HashSet<usize>,
    nodes_to_clear: HashSet<usize>,
    self_connected: HashSet<usize>,
}

impl NeuralSystem {
    pub const ENERGY_COST: f32 = 1e-6;

    pub fn new(
        neurons: &Vec<(bool, Neuron)>,
        connections: &Vec<Connection>,
        ns_shape: NsShape,
    ) -> Self {
        let mut nn_graph =
            StableGraph::<Neuron, f32>::with_capacity(ns_shape.n_neurons, connections.len());
        let mut memory_nodes = HashSet::<usize>::new();
        let out_start = ns_shape.input + ns_shape.hidden;

        for _ in 0..ns_shape.input {
            nn_graph.add_node(Neuron::new(0., Activation::Identity));
        }

        for (i, (is_memory, neuron)) in neurons.into_iter().enumerate() {
            if *is_memory {
                memory_nodes.insert(ns_shape.input + i);
            }
            nn_graph.add_node(*neuron);
        }

        for _ in out_start..ns_shape.n_neurons {
            nn_graph.add_node(Neuron::new(0., Activation::Tanh));
        }

        for c in connections.iter() {
            nn_graph.add_edge(NodeIndex::new(c.in_index), NodeIndex::new(c.out_index), c.w);
        }

        let reversed_graph = Reversed(&nn_graph);
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

        let nodes_to_clear =
            &(&HashSet::<usize>::from_iter(nodes.iter().cloned()) - &sources) - &memory_nodes;

        Self {
            ns_shape,
            nn_graph,
            nodes,
            sources,
            nodes_to_clear,
            self_connected,
        }
    }

    pub fn get_action(&mut self, input: Vec<f32>) -> Action {
        let mut rng = rand::thread_rng();

        let probas: Vec<_> = self
            .forward(&input)
            .iter()
            .enumerate()
            .map(|(i, p)| (i, if p.is_sign_negative() { 0. } else { *p }))
            .collect();

        let action_index = probas
            .choose_weighted(&mut rng, |(_, p)| *p)
            .unwrap_or(&(0, 0.))
            .0;

        Action::get(action_index)
    }

    pub fn forward(&mut self, input: &Vec<f32>) -> Vec<f32> {
        //set sensors, reset internal sources
        for index in self.sources.iter() {
            let source_neuron = self
                .nn_graph
                .node_weight_mut(NodeIndex::new(*index))
                .unwrap();
            source_neuron.value = if (0..self.ns_shape.input).contains(index) {
                input[*index]
            } else {
                source_neuron.w
            };
        }

        let mut self_connected_values =
            HashMap::<usize, f32>::with_capacity(self.self_connected.len());
        for index in self.self_connected.iter() {
            let value = self
                .nn_graph
                .node_weight(NodeIndex::new(*index))
                .unwrap()
                .value;
            self_connected_values.insert(*index, value);
        }

        for index in self.nodes_to_clear.iter() {
            let neuron = self
                .nn_graph
                .node_weight_mut(NodeIndex::new(*index))
                .unwrap();
            neuron.value = 0.;
        }

        for index in self.nodes.iter() {
            let node = NodeIndex::new(*index);

            let neighbors = self.nn_graph.neighbors_directed(node, Direction::Outgoing);

            if self.self_connected.contains(index) {
                let mut walk = neighbors.detach();
                let n_outgoing = neighbors.count();
                let mut outgoing = Vec::<(NodeIndex, f32)>::with_capacity(n_outgoing);
                let self_connected_value = *self_connected_values.get(index).unwrap();

                while let Some((edge, next_node)) = walk.next(&self.nn_graph) {
                    let edge_weight = *self.nn_graph.edge_weight(edge).unwrap();
                    outgoing.push((next_node, edge_weight));
                }

                let (self_out, other_conns): (Vec<(NodeIndex, f32)>, Vec<(NodeIndex, f32)>) =
                    outgoing.into_iter().partition(|x| x.0.index() == *index);

                //propagate to self
                let self_connected_neuron = self.nn_graph.node_weight_mut(node).unwrap();
                for conn in self_out.into_iter() {
                    self_connected_neuron.value += conn.1 * self_connected_value;
                }

                //fire self-connected
                let activated_value = self_connected_neuron.fire();

                //propagate to others
                for conn in other_conns.into_iter() {
                    let other_neuron = self.nn_graph.node_weight_mut(conn.0).unwrap();
                    other_neuron.value += conn.1 * activated_value;
                }
            } else {
                let mut walk = neighbors.detach();

                let neuron_value = if !self.sources.contains(index) {
                    self.nn_graph.node_weight_mut(node).unwrap().fire()
                } else {
                    self.nn_graph.node_weight(node).unwrap().value
                };

                while let Some((edge, next_node)) = walk.next(&self.nn_graph) {
                    let edge_weight = *self.nn_graph.edge_weight(edge).unwrap();
                    let next_neuron = self.nn_graph.node_weight_mut(next_node).unwrap();
                    next_neuron.value += edge_weight * neuron_value;
                }
            }
        }

        let out_start = self.ns_shape.input + self.ns_shape.hidden;
        let out_end = out_start + self.ns_shape.output;

        //get outputs
        let mut res = vec![0.; self.ns_shape.output];
        for index in out_start..out_end {
            let value = self
                .nn_graph
                .node_weight(NodeIndex::new(index))
                .unwrap()
                .value;
            res[index - out_start] = value;
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn test_output_and_pruning() {
        let ns_shape = NsShape::new(3, 2, 1);

        let neurons = vec![
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0., Activation::Tanh)),
        ];
        let connections = vec![
            Connection::new(1., ConnectionType::In, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::In, 1, 0).renumber(&ns_shape),
            Connection::new(0.3, ConnectionType::Out, 0, 0).renumber(&ns_shape),
            Connection::new(0.3, ConnectionType::In, 2, 1).renumber(&ns_shape),
        ];

        let mut ns = NeuralSystem::new(&neurons, &connections, ns_shape);

        assert_eq!(ns.nn_graph.node_count(), 5);
        assert_eq!(ns.sources, HashSet::<usize>::from_iter(vec![0, 1, 2]));

        let input = vec![0.5, 0.8];
        let out_inner = input.iter().sum::<f32>().tanh();
        let expected_output = (out_inner * 0.3).tanh();

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

        let neurons = vec![
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0.5, Activation::Tanh)),
        ];
        let mut connections = vec![
            Connection::new(1., ConnectionType::In, 0, 0).renumber(&ns_shape),
            Connection::new(0.6, ConnectionType::Internal, 1, 0).renumber(&ns_shape),
            Connection::new(0.4, ConnectionType::Internal, 2, 1).renumber(&ns_shape),
            Connection::new(0.5, ConnectionType::Out, 0, 0).renumber(&ns_shape),
        ];
        connections.shuffle(&mut rand::thread_rng());

        let mut ns = NeuralSystem::new(&neurons, &connections, ns_shape);
        assert_eq!(2, ns.sources.len());

        let input = vec![0.8];
        let weight_node_1: f32 = (0.5 as f32 * 0.4).tanh();
        let weight_node_0: f32 = (weight_node_1 * 0.6 + input[0]).tanh();
        let expected_output = (weight_node_0 * 0.5).tanh();

        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e9) as usize,
            (expected_output * 1e9) as usize
        );
    }

    #[test]
    fn test_self_connected_source() {
        let ns_shape = NsShape::new(1, 1, 1);

        let neurons = vec![(false, Neuron::new(0.5, Activation::Tanh))];
        let connections = vec![
            Connection::new(0.7, ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(0.3, ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(0.2, ConnectionType::Out, 0, 0).renumber(&ns_shape),
        ];

        let mut ns = NeuralSystem::new(&neurons, &connections, ns_shape);
        assert_eq!(2, ns.sources.len());

        let mut weight: f32 = 0.5;
        weight += 0.5 * 0.7;
        weight += 0.5 * 1.;
        weight += 0.5 * 0.3;
        weight = weight.tanh();
        let expected_output = (weight * 0.2).tanh();

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

        let neurons = vec![(false, Neuron::new(0., Activation::Tanh))];
        let connections = vec![
            Connection::new(1.2, ConnectionType::In, 0, 0).renumber(&ns_shape),
            Connection::new(0.9, ConnectionType::In, 1, 0).renumber(&ns_shape),
            Connection::new(0.7, ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(0.3, ConnectionType::Internal, 0, 0).renumber(&ns_shape),
            Connection::new(0.2, ConnectionType::Out, 0, 0).renumber(&ns_shape),
        ];

        let mut ns = NeuralSystem::new(&neurons, &connections, ns_shape);
        //set value to self-connected node
        ns.nn_graph
            .node_weight_mut(NodeIndex::new(2))
            .unwrap()
            .value = 0.74;

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
        let expected_output = (weight * 0.2).tanh();

        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e6) as usize,
            (expected_output * 1e6) as usize
        );
    }

    #[test]
    fn test_sources_and_pruning() {
        let ns_shape = NsShape::new(4, 4, 2);

        let neurons = vec![
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0., Activation::Tanh)),
            (false, Neuron::new(0., Activation::Tanh)),
        ];
        let mut connections = vec![
            //input to internal
            Connection::new(1., ConnectionType::In, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::In, 1, 0).renumber(&ns_shape),
            //input to output
            Connection::new(1., ConnectionType::InOut, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::InOut, 2, 0).renumber(&ns_shape),
            //self-connected
            Connection::new(1., ConnectionType::Internal, 1, 1).renumber(&ns_shape),
            //internal to internal
            Connection::new(1., ConnectionType::Internal, 1, 0).renumber(&ns_shape),
            //internal to output
            Connection::new(1., ConnectionType::Out, 0, 0).renumber(&ns_shape),
            Connection::new(1., ConnectionType::Out, 2, 1).renumber(&ns_shape),
            //input to internal unconnected
            Connection::new(1., ConnectionType::In, 3, 3).renumber(&ns_shape),
        ];
        //ensure connections ordering doesn't matter
        connections.shuffle(&mut rand::thread_rng());

        let ns = NeuralSystem::new(&neurons, &connections, ns_shape);

        //test sources
        assert_eq!(
            ns.sources,
            HashSet::<usize>::from_iter(vec![0, 1, 2, 3, ns_shape.input + 1, ns_shape.input + 2])
        );

        //test pruning
        assert_eq!(ns_shape.n_neurons - 1, ns.nn_graph.node_count());
        assert_eq!(connections.len() - 1, ns.nn_graph.edge_count());
    }

    #[test]
    fn test_memory_nodes() {
        let ns_shape = NsShape::new(1, 1, 1);

        let neurons = vec![(true, Neuron::new(0., Activation::Tanh))];
        let connections = vec![
            Connection::new(1.6, ConnectionType::In, 0, 0).renumber(&ns_shape),
            Connection::new(0.4, ConnectionType::Out, 0, 0).renumber(&ns_shape),
        ];

        let mut ns = NeuralSystem::new(&neurons, &connections, ns_shape);

        let input = vec![0.7];
        let memory_value = (0.7 * 1.6_f32).tanh();
        let expected_output: f32 = (((0.7 * 1.6_f32) + memory_value).tanh() * 0.4_f32).tanh();

        ns.forward(&input);
        let actual_output = *ns.forward(&input).first().unwrap();

        assert_eq!(
            (actual_output * 1e6) as usize,
            (expected_output * 1e6) as usize
        );
    }
}
