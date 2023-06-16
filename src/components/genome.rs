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

    pub fn replicate(&self, mut_p: f64, insert_p: f64, delete_p: f64) -> Self {
        let mut rng = rand::thread_rng();
        let mut child_genes = self.genes.clone();
        let genome_len = child_genes.len();

        if rng.gen_bool(mut_p) {
            let index = rng.gen_range(0..genome_len);
            child_genes[index] =
                child_genes[index].flip_bit(rng.gen_range(0..i32::BITS - 1) as usize);
        }

        if rng.gen_bool(insert_p) {
            let index = rng.gen_range(0..=genome_len);
            if index == genome_len {
                child_genes.push(rng.gen());
            } else {
                child_genes.insert(index, rng.gen());
            }
        }

        if rng.gen_bool(delete_p) {
            let index = rng.gen_range(0..genome_len);
            child_genes.remove(index);
        }

        Self { genes: child_genes }
    }

    pub fn get_distance(&self, other: &Self) -> f32 {
        let mut distance = (self.genes.len() as f32 - other.genes.len() as f32).abs();

        for (g1, g2) in self.genes.iter().zip(other.genes.iter()) {
            distance += g1.distance(*g2);
        }

        (distance / self.genes.len().max(other.genes.len()) as f32).clamp(0., 1.)
    }

    /**Sets genes to specified types*/
    pub fn set_gene_types(&mut self, n_connections: usize, n_neurons: usize) {
        let n_total = n_connections + n_neurons;
        if n_total > self.genes.len() {
            panic!("Required number of genes of specific types doesn't match genome length");
        }

        for i in 0..n_connections {
            self.genes[i].set_bit(31, 0);
        }

        for i in n_connections..n_total {
            self.genes[i].set_bit(31, 1);
            self.genes[i].set_bit(30, 1);
        }
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
            ((genome1.genes[1].get_conn_weight() - genome2.genes[1].get_conn_weight()).abs() + 2.)
                / 4.;

        let actual_distance = genome1.get_distance(&genome2);
        assert_eq!(actual_distance, expected_distance);
    }

    #[test]
    fn test_set_gene_types() {
        let mut genome = Genome::new(50);
        genome.set_gene_types(30, 20);

        assert_eq!(
            30,
            genome.genes.iter().filter(|g| g.is_connection()).count()
        );
        assert_eq!(20, genome.genes.iter().filter(|g| g.is_neuron()).count());
    }
}
