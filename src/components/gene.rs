use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

pub fn get_genome(len: usize) -> Vec<Gene> {
    let mut rng = rand::thread_rng();
    return (0..len).map(|_| rng.gen::<Gene>()).collect();
}

pub fn replicate_genome(genome: Vec<Gene>, mut_p: f64) -> Vec<Gene> {
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

fn flip_bit(n: i32, p: usize) -> i32 {
    n ^ (1 << p)
}

#[derive(Copy, Clone, PartialEq)]
/**Gene structure: 0tt-iiiiiii-ooooooo-wwwwwwwwwwwwwww, where t - connection types, i - input index, o - out index, w - weight*/
pub struct Gene(pub i32);

impl Gene {
    /**Get float from 15-bit weight value*/
    pub fn get_weightf(self) -> f32 {
        return (self.0 & 0x7fff) as f32 / (i16::MAX / 4) as f32;
    }

    /**Get 15-bit weight value*/
    pub fn get_weight(self) -> i32 {
        return (self.0 & 0x7fff) as i32;
    }

    /**Get 7-bit output index*/
    pub fn get_out_index(self) -> usize {
        return ((self.0 >> 15) & 0x7f) as usize;
    }

    /**Get 7-bit input index*/
    pub fn get_in_index(self) -> usize {
        return ((self.0 >> 22) & 0x7f) as usize;
    }

    /**Get 1-bit output type: 1 - sensor, 0 - neuron*/
    pub fn get_out_type(self) -> usize {
        return ((self.0 >> 29) & 1) as usize;
    }

    /**Get 1-bit input type: 1 - sensor, 0 - neuron*/
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
        let gene_value = 0b010_1001001_1111010_000011010000011; //1383925379
        let gene = Gene(gene_value);

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
}
