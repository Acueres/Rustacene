use super::*;
use bevy::prelude::Component;

#[derive(Component, Clone)]
pub struct Organism {
    pub genome: Vec<Gene>,
    pub age: usize,
    pub energy: f32,
}

impl Organism {
    pub fn new(energy: f32, genome_len: usize) -> Self {
        let genome: Vec<Gene> = get_genome(genome_len);
        Self {
            genome,
            age: 0,
            energy,
        }
    }

    pub fn replicate(self, mut_p: f32) -> Self {
        let new_genome = replicate_genome(self.genome, mut_p as f64);
        Self {
            genome: new_genome,
            age: 0,
            energy: 0.2,
        }
    }
}
