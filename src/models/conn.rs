use super::Gene;
use super::NsShape;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub w: f32,
    pub sensor_in: bool,
    pub sensor_out: bool,
    pub in_index: usize,
    pub out_index: usize,
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
        }
    }

    #[inline]
    pub fn from_gene(gene: Gene, ns_shape: &NsShape) -> Self {
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

        Connection::new(w, sensor_in, sensor_out, in_index, out_index)
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
            Connection::new(1., true, false, 0, 2),
            Connection::new(1., true, false, 1, 2),
            Connection::new(1., false, true, 2, 3),
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
            assert_eq!(test_conn.sensor_in, conn.sensor_in);
            assert_eq!(test_conn.sensor_out, conn.sensor_out);
        }
    }
}
