use super::Activation;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/**<b>Gene structure:</b>
 * Connection: 0-s-o-7i-7o-15w, where s - sensor, o - output, i - input index, o - out index, w - weight
 * Neuron: 11-2a-m-12i-15w, where a - activation type, m - memory neuron, i - neuron index, w - fire threshold or source weight */
#[derive(Copy, Clone, PartialEq)]
pub struct Gene(pub u32);

impl Gene {
    /**Check whether gene encodes a connection*/
    #[inline]
    pub fn is_connection(self) -> bool {
        ((self.0 >> 31) & 1) as i32 == 0
    }

    /**Check whether gene encodes a neuron*/
    #[inline]
    pub fn is_neuron(self) -> bool {
        ((self.0 >> 31) & 1) as i32 == 1 && ((self.0 >> 30) & 1) as i32 == 1
    }

    /**Get connection weight as f32 from 15-bit weight value*/
    #[inline]
    pub fn get_conn_weight(self) -> f32 {
        const SCALE: f32 = (i16::MAX / 4) as f32;
        let w = self.get_weight() - 0x4000;
        w as f32 / SCALE
    }

    /**Get neuronal firing threshold or source weight as f32 from 15-bit weight value*/
    #[inline]
    pub fn get_neuron_weight(self) -> f32 {
        let w = self.get_weight() as u32;
        w as f32 / i16::MAX as f32
    }

    /**Get 15-bit weight value*/
    #[inline]
    pub fn get_weight(self) -> i32 {
        (self.0 & 0x7fff) as i32
    }

    /**Get 7-bit output index*/
    #[inline]
    pub fn get_out_index(self) -> usize {
        ((self.0 >> 15) & 0x7f) as usize
    }

    /**Get 7-bit input index*/
    #[inline]
    pub fn get_in_index(self) -> usize {
        ((self.0 >> 22) & 0x7f) as usize
    }

    /**Get 1-bit output type: 1 - output, 0 - internal*/
    #[inline]
    pub fn get_out_type(self) -> usize {
        ((self.0 >> 29) & 1) as usize
    }

    /**Get 1-bit input type: 1 - sensor, 0 - internal*/
    #[inline]
    pub fn get_in_type(self) -> usize {
        ((self.0 >> 30) & 1) as usize
    }

    /**Get 2-bit neuron activation type*/
    #[inline]
    pub fn get_activation_type(self) -> Activation {
        Activation::get(((self.0 >> 28) & 3) as usize)
    }

    /**Check if neuron is memory type*/
    #[inline]
    pub fn is_memory(self) -> bool {
        ((self.0 >> 27) & 1) as usize == 1
    }

    /**Get 12-bit neuron index*/
    #[inline]
    pub fn get_neuron_index(self) -> usize {
        ((self.0 >> 15) & 0xfff) as usize
    }

    #[inline]
    pub fn flip_bit(self, pos: usize) -> Self {
        Self(self.0 ^ (1 << pos))
    }

    #[inline]
    pub fn set_bit(&mut self, pos: usize, value: usize) {
        if value == 1 {
            self.0 = self.0 | (1 << pos);
        } else {
            self.0 = self.0 & !(1 << pos);
        }
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
    fn test_connection_gene() {
        let value = 0b010_1001001_1111010_000011010000011;
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
    fn test_neuron_gene() {
        let value = 0b11_11_0_000010010001_000011010000011;
        let gene = Gene(value);

        let w = gene.get_weight();
        assert_eq!(w, 1667);

        let index = gene.get_neuron_index();
        assert_eq!(index, 145);

        let memory = gene.is_memory();
        assert_eq!(memory, false);

        let activation = gene.get_activation_type();
        assert_eq!(activation, Activation::Gaussian);
    }

    #[test]
    fn test_flip_bit() {
        let gene = Gene(0b010_1001001_1111010_000011010000011);

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

    #[test]
    fn test_set_bit() {
        let mut gene = Gene(0b100_1001001_1111010_000011010000011);
        gene.set_bit(31, 0);
        assert!(gene.is_connection());

        gene.set_bit(31, 1);
        gene.set_bit(30, 1);
        assert!(gene.is_neuron());
    }
}
