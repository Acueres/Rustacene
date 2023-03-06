/**Neural network shape */
#[derive(Clone, Copy)]
pub struct NsShape {
    pub input: usize,
    pub internal: usize,
    pub output: usize,
    pub n_neurons: usize,
}

impl NsShape {
    pub fn new(input: usize, internal: usize, output: usize) -> Self {
        Self {
            input,
            internal,
            output,
            n_neurons: input + internal + output,
        }
    }
}
