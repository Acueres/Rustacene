use super::conn::Connection;
use super::ns_shape::NsShape;
use crate::gene::Gene;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::Dfs;
use petgraph::Direction;
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]

pub struct NeuralSystem {
    ns_shape: NsShape,
    nn_graph: StableGraph<f32, f32>,
    sources: Vec<usize>,
}

impl NeuralSystem {
    pub fn init(genome: Vec<Gene>, n_connections: usize, ns_shape: NsShape) -> Self {
        let mut connections = Vec::<Connection>::with_capacity(n_connections);

        for gene in genome {
            connections.push(gene_to_conn(&gene, &ns_shape));
        }

        let mut nn_graph = get_nn_graph(&mut connections, ns_shape.n_neurons);
        let (connected_paths, unconnected_paths) = get_paths(&nn_graph, &ns_shape);
        let sources = get_sources(&connected_paths);
        let nodes_to_prune = get_prunable_nodes(&unconnected_paths, &ns_shape);

        prune_nodes(&mut nn_graph, &nodes_to_prune);

        let mut res = Self {
            ns_shape,
            nn_graph,
            sources,
        };
        res.init_sources();

        res
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

        let out_start = self.ns_shape.input + self.ns_shape.internal;
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
fn get_nn_graph(connections: &mut Vec<Connection>, n_nodes: usize) -> StableGraph<f32, f32> {
    connections.sort_by(|a, b| a.in_index.cmp(&b.in_index));

    let mut res = StableGraph::<f32, f32>::with_capacity(n_nodes, connections.len());

    for _ in 0..n_nodes {
        res.add_node(0.);
    }

    for c in connections.iter() {
        res.add_edge(NodeIndex::new(c.in_index), NodeIndex::new(c.out_index), c.w);
    }

    res
}

#[inline]
fn get_paths(
    graph: &StableGraph<f32, f32>,
    ns_shape: &NsShape,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let mut paths = Vec::<Vec<usize>>::with_capacity(ns_shape.input);
    let mut internal_nodes = HashSet::<usize>::with_capacity(ns_shape.internal);
    let out_start = ns_shape.input + ns_shape.internal;

    for start in graph.node_indices() {
        if start.index() >= out_start || !internal_nodes.insert(start.index()) {
            continue;
        }

        let mut dfs = Dfs::new(graph, start);
        let mut path = Vec::<usize>::new();

        while let Some(visited) = dfs.next(&graph) {
            internal_nodes.insert(visited.index());
            path.push(visited.index());
        }
        paths.push(path);
    }

    let (connected_paths, unconnected_paths): (_, Vec<_>) = paths
        .iter()
        .partition(|v| v.iter().any(|e| *e >= out_start));
    let connected_paths: Vec<_> = connected_paths
        .iter()
        .map(|x| x.to_owned().to_owned())
        .collect();
    let unconnected_paths: Vec<_> = unconnected_paths
        .iter()
        .map(|x| x.to_owned().to_owned())
        .collect();

    (connected_paths, unconnected_paths)
}

#[inline]
fn get_sources(paths: &Vec<Vec<usize>>) -> Vec<usize> {
    paths
        .iter()
        .map(|v| *v.first().unwrap())
        .collect::<Vec<usize>>()
}

#[inline]
fn get_prunable_nodes(paths: &Vec<Vec<usize>>, ns_shape: &NsShape) -> HashSet<usize> {
    paths
        .iter()
        .map(|x| x.iter().filter(|node| *node >= &ns_shape.input))
        .flatten()
        .cloned()
        .collect::<HashSet<usize>>()
}

#[inline]
fn prune_nodes(graph: &mut StableGraph<f32, f32>, nodes_to_prune: &HashSet<usize>) {
    for node in nodes_to_prune.iter() {
        graph.remove_node(NodeIndex::new(*node));
    }
}

#[inline]
fn gene_to_conn(gene: &Gene, ns_shape: &NsShape) -> Connection {
    let w = gene.get_weightf();
    let sensor_in = gene.get_in_type() == 1;
    let sensor_out = gene.get_out_type() == 1;

    let in_index = gene.get_in_index()
        % if sensor_in {
            ns_shape.input
        } else {
            ns_shape.internal
        };
    let in_index = renumber_in_index(in_index, sensor_in, ns_shape.input);

    let out_index = gene.get_out_index()
        % if sensor_out {
            ns_shape.output
        } else {
            ns_shape.internal
        };
    let out_index = renumber_out_index(
        out_index,
        sensor_out,
        ns_shape.input,
        ns_shape.input + ns_shape.internal,
    );

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
        let ns_shape = NsShape::new(2, 1, 1);
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
    fn test_nn_output() {
        let ns_shape = NsShape::new(2, 1, 1);

        let mut connections = vec![
            renumber_conn_indexes(&Connection::init(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, false, 1, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(0.3, false, true, 0, 0), &ns_shape),
        ];

        let graph = get_nn_graph(&mut connections, ns_shape.n_neurons);
        assert_eq!(graph.node_count(), ns_shape.n_neurons);
        assert_eq!(graph.edge_count(), connections.len());

        let input = vec![0.5, 0.8];
        let out_inner = input.iter().sum::<f32>().tanh(); //0.861723124
        let test_output: f32 = (out_inner * 0.3).tanh(); //0.252907842

        let mut ns = NeuralSystem {
            nn_graph: graph,
            sources: vec![0, 1],
            ns_shape,
        };
        let output = *ns.forward(&input).first().unwrap();

        assert_eq!((output * 1e9) as usize, (test_output * 1e9) as usize);
    }

    #[test]
    fn test_pruning() {
        let ns_shape = NsShape::new(1, 1, 1);

        let mut connections = vec![renumber_conn_indexes(
            &Connection::init(1., true, false, 0, 0),
            &ns_shape,
        )];

        let graph = get_nn_graph(&mut connections, ns_shape.n_neurons);
        let (connected_paths, unconnected_paths) = get_paths(&graph, &ns_shape);

        assert_eq!(0, connected_paths.len());
        assert_eq!(1, unconnected_paths.len());

        let nodes_to_prune = get_prunable_nodes(&unconnected_paths, &ns_shape);
        assert!(nodes_to_prune.contains(&ns_shape.input));
    }

    #[test]
    fn test_pruning_deep() {
        let ns_shape = NsShape::new(1, 3, 1);

        let mut connections = vec![
            renumber_conn_indexes(&Connection::init(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, false, 2, 1), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, false, 1, 0), &ns_shape),
        ];

        let graph = get_nn_graph(&mut connections, ns_shape.n_neurons);
        let (connected_paths, unconnected_paths) = get_paths(&graph, &ns_shape);

        assert_eq!(0, connected_paths.len());

        let nodes_to_prune = get_prunable_nodes(&unconnected_paths, &ns_shape);
        assert_eq!(3, nodes_to_prune.len());
        assert!(nodes_to_prune.contains(&ns_shape.input));
        assert!(nodes_to_prune.contains(&(ns_shape.input + 1)));
        assert!(nodes_to_prune.contains(&(ns_shape.input + 2)));
    }

    #[test]
    fn test_graph_integrated() {
        let ns_shape = NsShape::new(4, 4, 2);

        let mut connections = vec![
            //input to internal
            renumber_conn_indexes(&Connection::init(1., true, false, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, false, 1, 0), &ns_shape),
            //input to output
            renumber_conn_indexes(&Connection::init(1., true, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., true, true, 2, 0), &ns_shape),
            //self-connected
            renumber_conn_indexes(&Connection::init(1., false, false, 1, 1), &ns_shape),
            //internal to internal
            renumber_conn_indexes(&Connection::init(1., false, false, 1, 0), &ns_shape),
            //internal to output
            renumber_conn_indexes(&Connection::init(1., false, true, 0, 0), &ns_shape),
            renumber_conn_indexes(&Connection::init(1., false, true, 2, 1), &ns_shape),
            //input to internal unconnected
            renumber_conn_indexes(&Connection::init(1., true, false, 3, 3), &ns_shape),
        ];
        //ensure connections ordering does not matter
        connections.shuffle(&mut rand::thread_rng());

        let mut graph = get_nn_graph(&mut connections, ns_shape.n_neurons);
        let (connected_paths, unconnected_paths) = get_paths(&graph, &ns_shape);
        let sources = get_sources(&connected_paths);
        let nodes_to_prune = get_prunable_nodes(&unconnected_paths, &ns_shape);

        //test initial graph parameters
        assert_eq!(ns_shape.n_neurons, graph.node_count());
        assert_eq!(connections.len(), graph.edge_count());

        //test sources
        assert_eq!(5, sources.len());
        assert!(sources.contains(&0));
        assert!(sources.contains(&1));
        assert!(sources.contains(&2));
        assert!(sources.contains(&(ns_shape.input + 1)));
        assert!(sources.contains(&(ns_shape.input + 2)));

        //test prunable nodes
        assert_eq!(1, nodes_to_prune.len());
        assert!(nodes_to_prune.contains(&(ns_shape.input + 3)));

        prune_nodes(&mut graph, &nodes_to_prune);

        //test graph parameters after pruning
        assert_eq!(ns_shape.n_neurons - 1, graph.node_count());
        assert_eq!(connections.len() - 1, graph.edge_count());
    }

    #[inline]
    fn renumber_conn_indexes(conn: &Connection, ns_shape: &NsShape) -> Connection {
        let in_index = renumber_in_index(conn.in_index, conn.sensor_in, ns_shape.input);
        let out_index = renumber_out_index(
            conn.out_index,
            conn.sensor_out,
            ns_shape.input,
            ns_shape.input + ns_shape.internal,
        );

        Connection::init(conn.w, conn.sensor_in, conn.sensor_out, in_index, out_index)
    }
}
