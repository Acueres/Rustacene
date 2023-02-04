#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Connection {
    pub w: f32,
    pub sensor_in: bool,
    pub sensor_out: bool,
    pub in_index: usize,
    pub out_index: usize,
}

impl Connection {
    pub fn init(
        w: f32,
        sensor_in: bool,
        sensor_out: bool,
        in_index: usize,
        out_index: usize,
    ) -> Self {
        Self {
            w,
            sensor_in,
            sensor_out,
            in_index,
            out_index,
        }
    }
}
