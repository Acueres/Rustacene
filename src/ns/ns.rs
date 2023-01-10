use super::conn::Connection;
use crate::gene::Gene;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::Direction;
use petgraph::Graph;
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]

pub struct NeuralSystem {
    out_size: usize,
    ns_shape: (usize, usize, usize),
    nn_graph: Graph<f32, f32>,
    sources: Vec<usize>,
}

impl NeuralSystem {
    pub fn init(genome: Vec<Gene>, n_connections: usize, ns_shape: (usize, usize, usize)) -> Self {
        let mut connections = Vec::<Connection>::with_capacity(n_connections);

        for gene in genome {
            connections.push(gene_to_conn(&gene, &ns_shape));
        }

        let nn_graph = get_nn_graph(&connections, ns_shape.0 + ns_shape.1 + ns_shape.2);
        let sources = get_sources(&nn_graph, &ns_shape);

        let mut res = Self {
            out_size: ns_shape.2,
            ns_shape,
            nn_graph,
            sources,
        };
        res.init_sources();

        res
    }

    pub fn forward(&mut self, input: &Vec<f32>) -> Vec<f32> {
        let mut res = vec![0.0f32; self.out_size];

        //set inputs
        for input_index in 0..self.ns_shape.0 {
            *self
                .nn_graph
                .node_weight_mut(NodeIndex::new(input_index))
                .unwrap() = input[input_index];
        }

        let out_start = self.ns_shape.0 + self.ns_shape.1;
        let out_end = self.ns_shape.0 + self.ns_shape.1 + self.ns_shape.2;

        let mut visited = HashSet::<usize>::new();
        let mut nodes = VecDeque::<usize>::from_iter(self.sources.iter().cloned());

        while nodes.len() > 0 {
            let node_index = nodes.pop_front().unwrap();
            let mut node_out = *self
                .nn_graph
                .node_weight(NodeIndex::new(node_index))
                .unwrap();

            if (out_start..out_end).contains(&node_index) {
                res[node_index - out_start] = node_out.tanh();
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

    fn init_sources(&mut self) {
        for source_index in self.sources.iter() {
            *self
                .nn_graph
                .node_weight_mut(NodeIndex::new(*source_index))
                .unwrap() = 0.5;
        }
    }
}

#[inline]
fn get_nn_graph(connections: &Vec<Connection>, n_nodes: usize) -> Graph<f32, f32> {
    let mut res = Graph::<f32, f32>::new();
    res.reserve_exact_nodes(n_nodes);
    res.reserve_exact_edges(connections.len());
    for _ in 0..n_nodes {
        res.add_node(0.);
    }

    for c in connections.iter() {
        res.add_edge(NodeIndex::new(c.in_index), NodeIndex::new(c.out_index), c.w);
    }

    res
}

#[inline]
fn get_sources(graph: &Graph<f32, f32>, ns_shape: &(usize, usize, usize)) -> Vec<usize> {
    let mut paths = Vec::<Vec<usize>>::with_capacity(ns_shape.0);
    let mut internal_nodes = HashSet::<usize>::with_capacity(ns_shape.1);

    for start in graph.node_indices() {
        if !internal_nodes.insert(start.index()) {
            continue;
        }

        let mut dfs = Dfs::new(&graph, start);
        let mut path = Vec::<usize>::new();

        while let Some(visited) = dfs.next(&graph) {
            internal_nodes.insert(visited.index());
            path.push(visited.index());
        }
        paths.push(path);
        println!();
    }

    paths.iter().map(|v| v[0]).collect::<Vec<usize>>()
}

#[inline]
fn gene_to_conn(gene: &Gene, ns_shape: &(usize, usize, usize)) -> Connection {
    let w = gene.get_weightf();
    let sensor_in = gene.get_in_type() == 1;
    let sensor_out = gene.get_out_type() == 1;

    let in_index = gene.get_in_index() % if sensor_in { ns_shape.0 } else { ns_shape.1 };
    let in_index = renumber_in_index(in_index, sensor_in, ns_shape.0);

    let out_index = gene.get_out_index() % if sensor_out { ns_shape.2 } else { ns_shape.1 };
    let out_index = renumber_out_index(out_index, sensor_out, ns_shape.0, ns_shape.0 + ns_shape.1);

    Connection::init(w, sensor_in, sensor_out, in_index, out_index)
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

#[cfg(test)]
mod ns_tests {
    use super::*;
    use rand::seq::SliceRandom;

    #[test]
    fn test_gene_to_conn() {
        let ns_shape = (2, 1, 1);
        let test_conns = vec![
            Connection::init(1., true, false, 0, 2),
            Connection::init(1., true, false, 1, 2),
            Connection::init(1., false, true, 2, 3),
        ];

        let genes = vec![
            Gene(0b010_0000000_0000000_000011010000011),
            Gene(0b010_0000001_0000000_000011010000011),
            Gene(0b001_0000000_0000000_000011010000011),
        ];

        let conns: Vec<Connection> = genes
            .iter()
            .map(|gene| gene_to_conn(gene, &ns_shape))
            .collect();

        for (conn, test_conn) in conns.iter().zip(test_conns.iter()) {
            assert_eq!(test_conn.sensor_in, conn.sensor_in);
            assert_eq!(test_conn.sensor_out, conn.sensor_out);
        }
    }

    #[test]
    fn test_nn_graph() {
        let ns_shape = (2, 1, 1);
        let n_nodes = 4;
        let connections = vec![
            Connection::init(1., true, false, 0, 2),
            Connection::init(1., true, false, 1, 2),
            Connection::init(0.3, false, true, 2, 3),
        ];

        let graph = get_nn_graph(&connections, n_nodes);
        assert_eq!(graph.node_count(), n_nodes);
        assert_eq!(graph.edge_count(), connections.len());

        let input = vec![0.5, 0.8];
        let out_inner = input.iter().sum::<f32>().tanh(); //0.861723124
        let test_output: f32 = (out_inner * 0.3).tanh(); //0.252907842

        let mut ns = NeuralSystem {
            out_size: ns_shape.2,
            nn_graph: graph,
            sources: vec![0, 1],
            ns_shape,
        };
        let output = *ns.forward(&input).first().unwrap();

        assert_eq!((output * 1e9) as usize, (test_output * 1e9) as usize);
    }

    #[test]
    fn test_sources() {
        let ns_shape = (3, 3, 2);
        let n_nodes = 8;

        let mut connections = vec![
            renumber_conn_indexes(&Connection::init(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, true, 2, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, false, 1, 1), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, true, 2, 1), &ns_shape),
        ];
        connections.shuffle(&mut rand::thread_rng());

        let graph = get_nn_graph(&connections, n_nodes);
        let sources = get_sources(&graph, &ns_shape);

        assert_eq!(sources.len(), 5);
        assert!(sources.contains(&0));
        assert!(sources.contains(&1));
        assert!(sources.contains(&2));
        assert!(sources.contains(&4));
        assert!(sources.contains(&5));
    }

    #[inline]
    fn renumber_conn_indexes(conn: &Connection, ns_shape: &(usize, usize, usize)) -> Connection {
        let in_index = renumber_in_index(conn.in_index, conn.sensor_in, ns_shape.0);
        let out_index = renumber_out_index(
            conn.out_index,
            conn.sensor_out,
            ns_shape.0,
            ns_shape.0 + ns_shape.1,
        );

        Connection::init(conn.w, conn.sensor_in, conn.sensor_out, in_index, out_index)
    }
}
