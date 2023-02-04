use crate::dir::Dir;
use crate::gene::*;
use crate::ns::{NeuralSystem, NsShape};
use bevy::prelude::Component;
use rand::seq::SliceRandom;

#[derive(Component, Clone)]
pub struct Organism {
    pub genome: Vec<Gene>,
    pub age: usize,
    pub energy: f32,
    ns: NeuralSystem,
}

impl Organism {
    pub fn new(energy: f32, genome_len: usize, ns_shape: NsShape) -> Self {
        let genome: Vec<Gene> = get_genome(genome_len);
        let ns = NeuralSystem::init(genome.clone(), genome_len, ns_shape);
        Self {
            genome,
            age: 0,
            energy,
            ns,
        }
    }

    pub fn get_action(&mut self, input: Vec<f32>) -> Dir {
        let mut rng = rand::thread_rng();

        let probas = self.ns.forward(&input);
        let action_probas: Vec<(usize, f32)> = probas
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if p.is_sign_negative() {
                    (i, 0.0)
                } else {
                    (i, p.to_owned())
                }
            })
            .collect();
        let action_index = action_probas
            .choose_weighted(&mut rng, |(_, p)| *p)
            .unwrap_or(&(0, 0.))
            .0;

        Dir::get(action_index)
    }

    pub fn replicate(self, mut_p: f32, genome_len: usize, ns_shape: NsShape) -> Self {
        let new_genome = replicate_genome(self.genome, mut_p as f64);
        let ns = NeuralSystem::init(new_genome.clone(), genome_len, ns_shape);
        Self {
            genome: new_genome,
            age: 0,
            energy: 0.2,
            ns,
        }
    }
}
