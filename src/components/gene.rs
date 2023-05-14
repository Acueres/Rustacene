use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

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

    #[inline]
    pub fn flip_bit(self, pos: usize) -> Self {
        Self(self.0 ^ (1 << pos))
    }
}

impl Distribution<Gene> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gene {
        Gene(rng.gen())
    }
}

#[cfg(test)]
mod tests {
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
        let gene = Gene(0b010_1001001_1111010_000011010000011); //1383925379

        let gene_in_type_flipped = gene.flip_bit(30);
        let in_type = gene_in_type_flipped.get_in_type();
        assert_eq!(in_type, 0);

        let gene_out_type_flipped = gene.flip_bit(29);
        let out_type = gene_out_type_flipped.get_out_type();
        assert_eq!(out_type, 1);

        let gene_weight_flipped = gene.flip_bit(12);
        let weight = gene_weight_flipped.get_weight();
        assert_eq!(weight, 5763);
    }
}
