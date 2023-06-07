use super::Gene;
use super::NsShape;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ConnectionType {
    Internal,
    In,
    Out,
    InOut,
}

impl ConnectionType {
    #[inline]
    pub fn from_sensors(sensor_in: bool, sensor_out: bool) -> Self {
        if sensor_in && sensor_out {
            return ConnectionType::InOut;
        } else if sensor_in {
            return ConnectionType::In;
        } else if sensor_out {
            return ConnectionType::Out;
        }

        ConnectionType::Internal
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub w: f32,
    pub conn_type: ConnectionType,
    pub in_index: usize,
    pub out_index: usize,
}

impl Connection {
    pub fn new(w: f32, conn_type: ConnectionType, in_index: usize, out_index: usize) -> Self {
        Self {
            w,
            conn_type,
            in_index,
            out_index,
        }
    }

    #[inline]
    pub fn from_gene(gene: Gene, ns_shape: &NsShape) -> Self {
        let w = gene.get_conn_weight();
        let sensor_in = gene.get_in_type() == 1;
        let sensor_out = gene.get_out_type() == 1;

        let in_index = gene.get_in_index()
            % if sensor_in {
                ns_shape.input
            } else {
                ns_shape.hidden
            };

        let out_index = gene.get_out_index()
            % if sensor_out {
                ns_shape.output
            } else {
                ns_shape.hidden
            };

        let conn_type = ConnectionType::from_sensors(sensor_in, sensor_out);

        Connection::new(w, conn_type, in_index, out_index).renumber(&ns_shape)
    }

    #[inline]
    pub fn renumber(self, ns_shape: &NsShape) -> Self {
        let in_index = renumber_in_index(
            self.in_index,
            self.conn_type == ConnectionType::In || self.conn_type == ConnectionType::InOut,
            ns_shape.input,
        );
        let out_index = renumber_out_index(
            self.out_index,
            self.conn_type == ConnectionType::Out || self.conn_type == ConnectionType::InOut,
            ns_shape.input,
            ns_shape.input + ns_shape.hidden,
        );

        Connection::new(self.w, self.conn_type, in_index, out_index)
    }
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
mod tests {
    use super::*;

    #[test]
    fn test_gene_to_conn() {
        let ns_shape = NsShape::new(2, 1, 1);
        let test_conns = vec![
            Connection::new(1., ConnectionType::In, 0, 2),
            Connection::new(1., ConnectionType::In, 1, 2),
            Connection::new(1., ConnectionType::Out, 2, 3),
        ];

        let genes = vec![
            Gene(0b010_0000000_0000000_000011010000011),
            Gene(0b010_0000001_0000000_000011010000011),
            Gene(0b001_0000000_0000000_000011010000011),
        ];

        let conns: Vec<Connection> = genes
            .iter()
            .map(|gene| Connection::from_gene(*gene, &ns_shape))
            .collect();

        for (conn, test_conn) in conns.iter().zip(test_conns.iter()) {
            assert_eq!(test_conn.conn_type, conn.conn_type);
        }
    }
}
