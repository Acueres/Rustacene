use super::{Activation, Gene};

#[derive(Copy, Clone, PartialEq)]
pub struct Neuron {
    pub w: f32,
    pub value: f32,
    activation: Activation,
}

impl Neuron {
    #[inline]
    pub fn new(w: f32, activation: Activation) -> Self {
        Self {
            w,
            value: 0.,
            activation,
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
                activation: gene.get_activation_type(),
            },
        )
    }

    #[inline]
    pub fn fire(&mut self) -> f32 {
        if self.value.abs() > self.w {
            self.value = match self.activation {
                Activation::Tanh => self.value.tanh(),
                Activation::Sigmoid => sigmoid(self.value),
                Activation::ReLU => relu(self.value),
                Activation::Gaussian => gaussian(self.value),
                Activation::None => self.value,
            };

            return self.value;
        }
        return 0.;
    }
}

#[inline]
fn sigmoid(v: f32) -> f32 {
    1. / (1. + (-v).exp())
}

#[inline]
fn relu(v: f32) -> f32 {
    if v <= 0. {
        0.
    } else {
        1.
    }
}

#[inline]
fn gaussian(v: f32) -> f32 {
    (-v * v).exp()
}
