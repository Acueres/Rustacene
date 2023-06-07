pub type ActivationFn = fn(f32) -> f32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activation {
    Tanh,
    Sigmoid,
    ReLU,
    Gaussian,
    Identity,
}

impl Activation {
    pub fn get(val: usize) -> Self {
        match val {
            0 => Self::Tanh,
            1 => Self::Sigmoid,
            2 => Self::ReLU,
            3 => Self::Gaussian,
            4 => Self::Identity,
            _ => panic!("Activation range error"),
        }
    }

    pub fn get_fn(self) -> ActivationFn {
        match self {
            Self::Tanh => tanh,
            Self::Sigmoid => sigmoid,
            Self::ReLU => relu,
            Self::Gaussian => gaussian,
            Self::Identity => identity,
        }
    }
}

fn tanh(v: f32) -> f32 {
    v.tanh()
}

fn sigmoid(v: f32) -> f32 {
    1. / (1. + (-v).exp())
}

fn relu(v: f32) -> f32 {
    if v <= 0. {
        0.
    } else {
        1.
    }
}

fn gaussian(v: f32) -> f32 {
    (-v * v).exp()
}

fn identity(v: f32) -> f32 {
    v
}
