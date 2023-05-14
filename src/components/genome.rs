use super::Gene;
use rand::Rng;

#[derive(Clone, PartialEq)]
pub struct Genome {
    data: Vec<Gene>,
}

impl Genome {
    pub fn new(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            data: (0..len).map(|_| rng.gen::<Gene>()).collect(),
        }
    }

    #[cfg(test)]
    pub fn from(genes: Vec<Gene>) -> Self {
        Self { data: genes }
    }

    pub fn replicate(&self, mut_p: f64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            data: self
                .data
                .iter()
                .map(|g| {
                    if rng.gen_bool(mut_p) {
                        g.flip_bit(rng.gen_range(0..i32::BITS - 1) as usize)
                    } else {
                        g.to_owned()
                    }
                })
                .collect(),
        }
    }

    pub fn get_distance(&self, other: &Self) -> f32 {
        let mut distance = 0.;
        for (g1, g2) in self.data.iter().zip(other.data.iter()) {
            if g1.get_in_type() == g2.get_in_type()
                && g1.get_out_type() == g2.get_out_type()
                && g1.get_in_index() == g2.get_in_index()
                && g1.get_out_index() == g2.get_out_index()
            {
                distance += (g1.get_weightf() - g2.get_weightf()).abs();
            } else {
                distance += 1.;
            }
        }

        (distance / other.data.len() as f32).clamp(0., 1.)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Gene> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_distance() {
        let genome1 = Genome::from(vec![
            Gene(0b010_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111110_000011010000011),
        ]);

        let genome2 = Genome::from(vec![
            Gene(0b010_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111010_000010010000011),
            Gene(0b011_1001011_1111110_000011010000011),
        ]);

        //first genes are equal, second genes differ in weights, third genes are disjoint
        let expected_distance =
            ((genome1.data[1].get_weightf() - genome2.data[1].get_weightf()).abs() + 1.) / 3.;

        let actual_distance = genome1.get_distance(&genome2);
        assert_eq!(actual_distance, expected_distance);
    }
}
