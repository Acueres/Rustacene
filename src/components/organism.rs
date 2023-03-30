use super::*;
use bevy::prelude::Component;

const REPLICATION_COST: f32 = 0.2;

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

    pub fn replicate(&mut self, mut_p: f32) -> Self {
        let new_genome = replicate_genome(&self.genome, mut_p as f64);
        self.energy -= REPLICATION_COST;
        Self {
            genome: new_genome,
            age: 0,
            energy: REPLICATION_COST,
        }
    }

    pub fn add_energy(&mut self, quantity: f32) {
        self.energy += quantity;
    }

    pub fn sub_energy(&mut self, quantity: f32) {
        self.energy -= quantity;
    }
}
