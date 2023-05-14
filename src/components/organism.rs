use super::*;
use bevy::prelude::Component;

const REPLICATION_COST: f32 = 0.2;

#[derive(Component, Clone)]
pub struct Organism {
    pub genome: Genome,
    pub species: usize,
    pub age: usize,
    pub energy: f32,
}

impl Organism {
    pub fn new(energy: f32, genome_len: usize) -> Self {
        let genome = Genome::new(genome_len);
        Self {
            genome,
            species: 0,
            age: 0,
            energy,
        }
    }

    pub fn replicate(&mut self, mut_p: f32) -> Self {
        let new_genome = self.genome.replicate(mut_p as f64);
        self.energy -= REPLICATION_COST;
        Self {
            genome: new_genome,
            species: self.species,
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
