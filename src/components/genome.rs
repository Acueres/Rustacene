use super::Gene;
use rand::Rng;

#[derive(Clone, PartialEq)]
pub struct Genome {
    genes: Vec<Gene>,
}

impl Genome {
    pub fn new(len: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            genes: (0..len).map(|_| rng.gen::<Gene>()).collect(),
        }
    }

    #[cfg(test)]
    pub fn from(genes: Vec<Gene>) -> Self {
        Self { genes }
    }

    pub fn replicate(&self, mut_p: f64, insert_p: f64) -> Self {
        let mut rng = rand::thread_rng();
        let mut child_genes = self.genes.clone();

        if !rng.gen_bool(mut_p) {
            return Self { genes: child_genes };
        }

        let genome_len = child_genes.len();
        let index = rng.gen_range(0..=genome_len);
        let insert = rng.gen_bool(insert_p);
        let append = insert && index == genome_len;

        if append {
            child_genes.push(rng.gen());
        } else if insert {
            child_genes.insert(index, rng.gen());
        } else {
            let index = index.clamp(0, genome_len - 1);
            child_genes[index] =
                child_genes[index].flip_bit(rng.gen_range(0..i32::BITS - 1) as usize);
        }

        Self { genes: child_genes }
    }

    pub fn get_distance(&self, other: &Self) -> f32 {
        let mut distance = (self.genes.len() as f32 - other.genes.len() as f32).abs();

        for (g1, g2) in self.genes.iter().zip(other.genes.iter()) {
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

        (distance / self.genes.len().max(other.genes.len()) as f32).clamp(0., 1.)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Gene> {
        self.genes.iter()
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
            Gene(0b011_1001111_1111110_000011010010011),
        ]);

        //first genes are equal, second genes differ in weights, third genes are disjoint, fourth genes are excessive
        let expected_distance =
            ((genome1.genes[1].get_weightf() - genome2.genes[1].get_weightf()).abs() + 2.) / 4.;

        let actual_distance = genome1.get_distance(&genome2);
        assert_eq!(actual_distance, expected_distance);
    }
}
