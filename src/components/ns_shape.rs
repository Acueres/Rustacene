/**Neural network shape */
#[derive(Clone, Copy)]
pub struct NsShape {
    pub input: usize,
    pub hidden: usize,
    pub output: usize,
    pub n_neurons: usize,
}

impl NsShape {
    pub fn new(input: usize, hidden: usize, output: usize) -> Self {
        Self {
            input,
            hidden,
            output,
            n_neurons: input + hidden + output,
        }
    }
}
