use super::*;
use bevy::prelude::Component;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Component, Clone)]
pub struct Organism {
    pub genome: Vec<Gene>,
    pub age: usize,
    pub energy: f32,
    pub direction: Dir,
    ns: NeuralSystem,
}

impl Organism {
    pub fn new(energy: f32, genome_len: usize, ns_shape: NsShape) -> Self {
        let mut rng = rand::thread_rng();
        let genome: Vec<Gene> = get_genome(genome_len);
        let ns = NeuralSystem::init(genome.clone(), genome_len, ns_shape);
        Self {
            genome,
            age: 0,
            energy,
            direction: rng.gen(),
            ns,
        }
    }

    #[inline]
    pub fn get_action(&mut self, input: Vec<f32>) -> Action {
        let mut rng = rand::thread_rng();

        let probas: Vec<_> = self
            .ns
            .forward(&input)
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.to_owned()))
            .collect();

        let action_index = probas
            .choose_weighted(&mut rng, |(_, p)| *p)
            .unwrap_or(&(0, 0.))
            .0;

        Action::get(action_index)
    }

    pub fn replicate(self, mut_p: f32, genome_len: usize, ns_shape: NsShape) -> Self {
        let mut rng = rand::thread_rng();
        let new_genome = replicate_genome(self.genome, mut_p as f64);
        let ns = NeuralSystem::init(new_genome.clone(), genome_len, ns_shape);
        Self {
            genome: new_genome,
            age: 0,
            energy: 0.2,
            direction: rng.gen(),
            ns,
        }
    }
}
