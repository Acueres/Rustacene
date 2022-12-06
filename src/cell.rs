use bevy::prelude::Component;
use rand::Rng;

#[derive(Component, Clone)]
pub struct Cell {
    pub genome: Vec<f32>,
    pub age: usize,
}

impl Cell {
    pub fn replicate(self, mut_proba: f32) -> Self {
        let mut rng = rand::thread_rng();
        let new_genome: Vec<f32> = self
            .genome
            .iter()
            .map(|f| {
                if rng.gen_bool(mut_proba as f64) {
                    rng.gen_range(-1.0..1.)
                } else {
                    f.to_owned()
                }
            })
            .collect();

        Self {
            genome: new_genome,
            age: 0,
        }
    }
}
