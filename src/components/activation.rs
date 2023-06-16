#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activation {
    None,
    Tanh,
    Sigmoid,
    ReLU,
    Gaussian,
}

impl Activation {
    pub fn get(val: usize) -> Self {
        match val {
            0 => Self::Tanh,
            1 => Self::Sigmoid,
            2 => Self::ReLU,
            3 => Self::Gaussian,
            4 => Self::None,
            _ => panic!("Activation range error"),
        }
    }
}
