use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashSet;

#[inline]
pub fn get_genome(len: usize) -> Vec<Gene> {
    let mut rng = rand::thread_rng();
    return (0..len).map(|_| rng.gen::<Gene>()).collect();
}

#[inline]
pub fn replicate_genome(genome: &Vec<Gene>, mut_p: f64) -> Vec<Gene> {
    let mut rng = rand::thread_rng();
    genome
        .iter()
        .map(|g| {
            if rng.gen_bool(mut_p) {
                Gene(flip_bit(g.0, rng.gen_range(0..i32::BITS - 1) as usize))
            } else {
                g.to_owned()
            }
        })
        .collect()
}

pub fn cluster_species(genomes: &Vec<&Vec<Gene>>, threshold: f32) -> (Vec<usize>, HashSet<usize>) {
    let mut species_data = vec![0; genomes.len()];
    let mut unassigned = HashSet::<usize>::from_iter(0..genomes.len());
    let mut species = HashSet::<usize>::new();
    let mut species_counter = 0;

    for (i, genome) in genomes.iter().enumerate() {
        if !unassigned.remove(&i) {
            continue;
        }

        species_data[i] = species_counter;

        let species_members: Vec<_> = unassigned
            .iter()
            .map(|index| (*index, get_genetic_distance(genome, genomes[*index])))
            .filter(|(_, d)| *d < threshold)
            .map(|(index, _)| index)
            .collect();

        for index in species_members.iter() {
            species_data[*index] = species_counter;
        }

        unassigned = &unassigned - &HashSet::<usize>::from_iter(species_members.iter().cloned());

        species.insert(species_counter);
        species_counter += 1;
    }

    (species_data, species)
}

#[inline]
pub fn get_genetic_distance(genome1: &Vec<Gene>, genome2: &Vec<Gene>) -> f32 {
    let mut distance = 0.;
    for (g1, g2) in genome1.iter().zip(genome2.iter()) {
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

    (distance / genome1.len() as f32).clamp(0., 1.)
}

#[inline]
fn flip_bit(n: i32, p: usize) -> i32 {
    n ^ (1 << p)
}

#[derive(Copy, Clone, PartialEq)]
/**Gene structure: 0so-7i-7o-15w, where s - sensor, o - output, i - input index, o - out index, w - weight*/
pub struct Gene(pub i32);

impl Gene {
    /**Get float from 15-bit weight value*/
    #[inline]
    pub fn get_weightf(self) -> f32 {
        let w = self.get_weight() - 0x4000;
        return w as f32 / (i16::MAX / 4) as f32;
    }

    /**Get 15-bit weight value*/
    #[inline]
    pub fn get_weight(self) -> i32 {
        return (self.0 & 0x7fff) as i32;
    }

    /**Get 7-bit output index*/
    #[inline]
    pub fn get_out_index(self) -> usize {
        return ((self.0 >> 15) & 0x7f) as usize;
    }

    /**Get 7-bit input index*/
    #[inline]
    pub fn get_in_index(self) -> usize {
        return ((self.0 >> 22) & 0x7f) as usize;
    }

    /**Get 1-bit output type: 1 - output, 0 - internal*/
    #[inline]
    pub fn get_out_type(self) -> usize {
        return ((self.0 >> 29) & 1) as usize;
    }

    /**Get 1-bit input type: 1 - sensor, 0 - internal*/
    #[inline]
    pub fn get_in_type(self) -> usize {
        return ((self.0 >> 30) & 1) as usize;
    }
}

impl Distribution<Gene> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gene {
        Gene(rng.gen())
    }
}

#[cfg(test)]
mod gene_tests {
    use super::*;

    #[test]
    fn test_gene() {
        let value = 0b010_1001001_1111010_000011010000011; //1383925379
        let gene = Gene(value);

        let w = gene.get_weight();
        assert_eq!(w, 1667);

        let out_i = gene.get_out_index();
        assert_eq!(out_i, 122);

        let in_i = gene.get_in_index();
        assert_eq!(in_i, 73);

        let out_t = gene.get_out_type();
        assert_eq!(out_t, 0);

        let in_t = gene.get_in_type();
        assert_eq!(in_t, 1);
    }

    #[test]
    fn test_bit_flip() {
        let value = 0b010_1001001_1111010_000011010000011; //1383925379

        let in_type_flipped = flip_bit(value, 30);
        let gene = Gene(in_type_flipped);
        let res = gene.get_in_type();
        assert_eq!(res, 0);

        let out_type_flipped = flip_bit(value, 29);
        let gene = Gene(out_type_flipped);
        let res = gene.get_out_type();
        assert_eq!(res, 1);

        let weight_flipped = flip_bit(value, 12);
        let gene = Gene(weight_flipped);
        let res = gene.get_weight();
        assert_eq!(res, 5763);
    }

    #[test]
    fn test_genome_distance() {
        let genome1 = vec![
            Gene(0b010_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111110_000011010000011),
        ];

        let genome2 = vec![
            Gene(0b010_1001001_1111010_000011010000011),
            Gene(0b011_1001001_1111010_000010010000011),
            Gene(0b011_1001011_1111110_000011010000011),
        ];

        //first genes are equal, second genes differ in weights, third genes are disjoint
        let expected_distance =
            ((genome1[1].get_weightf() - genome2[1].get_weightf()).abs() + 1.) / 3.;

        let actual_distance = get_genetic_distance(&genome1, &genome2);
        assert_eq!(actual_distance, expected_distance);
    }

    #[test]
    fn test_speciation() {
        let genomes = vec![
            //species 0: two equal genomes
            vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ],
            vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ],
            //species 0: slight weight difference
            vec![
                Gene(0b010_1001001_1111010_000011010000111),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ],
            //species 1: ~30% difference
            vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000010010000011),
                Gene(0b011_1001011_1111110_000011010000011),
            ],
            //species 1: slight weight difference
            vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000010010010011),
                Gene(0b011_1001011_1111110_000011010000011),
            ],
            //species 2
            vec![
                Gene(0b011_1001001_1011010_010011010000011),
                Gene(0b001_1001001_1011010_001010010010011),
                Gene(0b010_1001011_1111010_010011010000011),
            ],
        ];

        let (clustered, species) = cluster_species(&(genomes.iter().map(|g| g).collect()), 1e-1);
        assert_eq!(3, species.len());
        assert_eq!(vec![0, 0, 0, 1, 1, 2], clustered);
    }
}
