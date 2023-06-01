use super::*;
use bevy::prelude::Component;

const MINIMUM_REPLICATION_ENERGY: f32 = 0.25;
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

    #[inline]
    pub fn replicate(&mut self, mut_p: f64, insert_p: f64, delete_p: f64) -> Self {
        self.energy -= REPLICATION_COST;
        Self {
            genome: self.genome.replicate(mut_p, insert_p, delete_p),
            species: self.species,
            age: 0,
            energy: REPLICATION_COST,
        }
    }

    #[inline]
    pub fn add_energy(&mut self, quantity: f32) {
        self.energy += quantity;
    }

    #[inline]
    pub fn sub_energy(&mut self, quantity: f32) {
        self.energy -= quantity;
    }

    #[inline]
    pub fn can_replicate(&self) -> bool {
        self.energy > MINIMUM_REPLICATION_ENERGY
    }
}
