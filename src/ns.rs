use crate::gene::Gene;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]

pub struct NeuralSystem {
    out_size: usize,
    direct_connections: Vec<Connection>,
    input_connections: Vec<Connection>,
    internal_connections: Vec<Connection>,
    out_connections: Vec<Connection>,
    internal_values_ids: HashSet<usize>,
}

impl NeuralSystem {
    pub fn init(genome: Vec<Gene>, ns_shape: (usize, usize, usize)) -> Self {
        let n_connections = ns_shape.0 + ns_shape.1 + ns_shape.2;
        let mut connections = Vec::<Connection>::with_capacity(n_connections);

        for gene in genome {
            let w = gene.get_weightf();
            let sensor_in = gene.get_in_type() == 1;
            let sensor_out = gene.get_out_type() == 1;
            let in_index = gene.get_in_index() % if sensor_in { ns_shape.0 } else { ns_shape.1 };
            let out_index = gene.get_out_index() % if sensor_out { ns_shape.2 } else { ns_shape.1 };

            connections.push(Connection::new(
                w, sensor_in, sensor_out, in_index, out_index,
            ));
        }

        let (
            direct_connections,
            input_connections,
            internal_connections,
            out_connections,
            internal_values_map,
        ) = process_connections(&connections);

        Self {
            out_size: ns_shape.2,
            direct_connections,
            input_connections,
            internal_connections,
            out_connections,
            internal_values_ids: internal_values_map,
        }
    }

    pub fn forward(&self, input: Vec<f32>) -> Vec<f32> {
        let mut res = vec![0.; self.out_size];
        let mut self_connected = Vec::<&Connection>::new();
        let mut values_map = HashMap::<usize, f32>::from_iter(
            self.internal_values_ids
                .iter()
                .map(|id| (id.to_owned(), 0.)),
        );

        for c in self.direct_connections.iter() {
            res[c.out_index] += c.w * input[c.in_index];
        }
        for c in self.input_connections.iter() {
            *values_map.get_mut(&c.out_index).unwrap() += c.w * input[c.in_index];
        }
        for c in self.internal_connections.iter() {
            if !c.self_connected {
                *values_map.get_mut(&c.out_index).unwrap() += c.w * input[c.in_index];
            } else {
                self_connected.push(c);
            }
        }
        for c in self_connected {
            let mut sum = *values_map.get(&c.out_index).unwrap();
            sum += c.w * sum;
            sum = sum.tanh();
            *values_map.get_mut(&c.out_index).unwrap() = sum;
        }
        for c in self.out_connections.iter() {
            if values_map.contains_key(&c.in_index) {
                res[c.out_index] += c.w * values_map[&c.in_index];
            } else {
                res[c.out_index] += c.w;
            }
        }
        res.iter().map(|x| x.tanh()).collect::<Vec<f32>>()
    }
}

//*Divides connections into layers, prunes unconnected neurons and internal connections with depth of 1*/
fn process_connections(
    connections: &Vec<Connection>,
) -> (
    Vec<Connection>,
    Vec<Connection>,
    Vec<Connection>,
    Vec<Connection>,
    HashSet<usize>,
) {
    let direct_connections = connections
        .iter()
        .filter(|c| c.sensor_in && c.sensor_out)
        .cloned()
        .collect::<Vec<Connection>>();

    let out_connections = connections
        .iter()
        .filter(|c| !c.sensor_in && c.sensor_out)
        .cloned()
        .collect::<Vec<Connection>>();

    let internal_connections = connections
        .iter()
        .filter(|c| {
            !(c.sensor_in || c.sensor_out)
                && out_connections.iter().any(|x| c.out_index == x.in_index)
        })
        .cloned()
        .collect::<Vec<Connection>>();

    let input_connections = connections
        .iter()
        .filter(|c| {
            (c.sensor_in && !c.sensor_out)
                && (internal_connections
                    .iter()
                    .any(|x| c.out_index == x.in_index)
                    || out_connections.iter().any(|x| c.out_index == x.in_index))
        })
        .cloned()
        .collect::<Vec<Connection>>();

    let mut internal_values_ids = HashSet::<usize>::new();
    for c in input_connections.iter().chain(internal_connections.iter()) {
        internal_values_ids.insert(c.out_index);
    }

    return (
        direct_connections,
        input_connections,
        internal_connections,
        out_connections,
        internal_values_ids,
    );
}

#[derive(Copy, Clone, PartialEq)]
pub struct Connection {
    pub w: f32,
    pub sensor_in: bool,
    pub sensor_out: bool,
    pub in_index: usize,
    pub out_index: usize,
    pub self_connected: bool,
}

impl Connection {
    pub fn new(
        w: f32,
        sensor_in: bool,
        sensor_out: bool,
        in_index: usize,
        out_index: usize,
    ) -> Self {
        Self {
            w,
            sensor_in,
            sensor_out,
            in_index,
            out_index,
            self_connected: !(sensor_in || sensor_out) && in_index == out_index,
        }
    }
}

#[cfg(test)]
mod ns_tests {
    use super::*;

    #[test]
    fn test_connections() {
        let connections = vec![
            //direct 0 to 0
            Connection::new(1., true, true, 0, 0),
            //in 1 to internal 1
            Connection::new(1., true, false, 1, 1),
            //internal 0 to internal 1
            Connection::new(1., false, false, 0, 1),
            //internal 1 to internal 1
            Connection::new(1., false, false, 1, 1),
            //internal 1 to out 1
            Connection::new(1., false, true, 1, 1),
            //internal 4 to out 4
            Connection::new(1., false, true, 4, 4),
            //in 1 to internal 2: prune unconnected
            Connection::new(1., true, false, 1, 2),
            //internal 3 to internal 3: prune unconnected
            Connection::new(1., false, false, 3, 3),
            //internal 0 to internal 0: prune connection with depth > 1
            Connection::new(1., false, false, 0, 0),
        ];

        let (
            direct_connections,
            input_connections,
            internal_connections,
            out_connections,
            internal_values_map,
        ) = process_connections(&connections);

        assert!(direct_connections.len() == 1);
        assert!(input_connections.len() == 1);
        assert!(internal_connections.len() == 2);
        assert!(out_connections.len() == 2);
        assert!(internal_values_map.len() == 1)
    }
}
