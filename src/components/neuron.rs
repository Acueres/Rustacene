use super::{Activation, ActivationFn, Gene};

#[derive(Copy, Clone, PartialEq)]
pub struct Neuron {
    pub w: f32,
    pub value: f32,
    activation: ActivationFn,
}

impl Neuron {
    #[inline]
    pub fn new(w: f32, activation: Activation) -> Self {
        Self {
            w,
            value: 0.,
            activation: activation.get_fn(),
        }
    }

    #[inline]
    pub fn from_gene(gene: Gene) -> (usize, bool, Self) {
        (
            gene.get_neuron_index(),
            gene.is_memory(),
            Self {
                w: gene.get_neuron_weight(),
                value: 0.,
                activation: gene.get_activation_type().get_fn(),
            },
        )
    }

    #[inline]
    pub fn fire(&mut self) -> f32 {
        if self.value.abs() > self.w {
            self.value = (self.activation)(self.value);
            return self.value;
        }
        return 0.;
    }
}
