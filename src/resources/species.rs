use crate::components::Genome;
use bevy::prelude::Color;
use bevy::prelude::Resource;
use rand::Rng;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Clone)]
pub struct Species {
    species: HashSet<usize>,
    colors: HashSet<(u8, u8, u8)>,
    color_map: HashMap<usize, Color>,
}

impl Species {
    pub fn new(species: HashSet<usize>) -> Self {
        let mut rng = rand::thread_rng();
        let mut colors = HashSet::<(u8, u8, u8)>::with_capacity(species.len());
        let n_species = species.len();

        while colors.len() < n_species {
            colors.insert((
                rng.gen_range(0..u8::MAX),
                rng.gen_range(0..u8::MAX),
                rng.gen_range(0..u8::MAX),
            ));
        }

        let mut color_data = HashMap::<usize, Color>::with_capacity(n_species);
        for (s, c) in species.iter().zip(colors.iter()) {
            color_data.insert(*s, Color::rgb_u8(c.0, c.1, c.2));
        }

        Self {
            species,
            colors,
            color_map: color_data,
        }
    }

    pub fn cluster(genomes: &Vec<&Genome>, threshold: f32) -> (Self, Vec<usize>) {
        let mut assigned_species = vec![0; genomes.len()];
        let mut unassigned = HashSet::<usize>::from_iter(0..genomes.len());
        let mut species = HashSet::<usize>::new();
        let mut species_counter = 0;

        for (i, genome) in genomes.iter().enumerate() {
            if !unassigned.remove(&i) {
                continue;
            }

            assigned_species[i] = species_counter;

            let species_members: Vec<_> = unassigned
                .iter()
                .map(|index| (*index, genome.get_distance(genomes[*index])))
                .filter(|(_, d)| *d < threshold)
                .map(|(index, _)| index)
                .collect();

            for index in species_members.iter() {
                assigned_species[*index] = species_counter;
            }

            unassigned =
                &unassigned - &HashSet::<usize>::from_iter(species_members.iter().cloned());

            species.insert(species_counter);
            species_counter += 1;
        }

        (Self::new(species), assigned_species)
    }

    #[cfg(test)]
    #[inline]
    pub fn len(&self) -> usize {
        self.species.len()
    }

    #[inline]
    pub fn next_species(&self) -> usize {
        self.species.len()
    }

    #[inline]
    pub fn add_species(&mut self, species: usize) {
        if self.species.insert(species) {
            let mut rng = rand::thread_rng();
            loop {
                let color = (
                    rng.gen_range(0..u8::MAX),
                    rng.gen_range(0..u8::MAX),
                    rng.gen_range(0..u8::MAX),
                );
                if self.colors.insert(color) {
                    self.color_map
                        .insert(species, Color::rgb_u8(color.0, color.1, color.2));
                    break;
                }
            }
        }
    }

    #[inline]
    pub fn get_color(&self, species: usize) -> Color {
        *self
            .color_map
            .get(&species)
            .unwrap_or_else(|| &Color::WHITE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Gene;

    #[test]
    fn test_speciation() {
        let genomes = vec![
            //species 0: two equal genomes
            Genome::from(vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ]),
            Genome::from(vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ]),
            //species 0: slight weight difference
            Genome::from(vec![
                Gene(0b010_1001001_1111010_000011010000111),
                Gene(0b011_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111110_000011010000011),
            ]),
            //species 1: ~30% difference
            Genome::from(vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000010010000011),
                Gene(0b011_1001011_1111110_000011010000011),
            ]),
            //species 1: slight weight difference
            Genome::from(vec![
                Gene(0b010_1001001_1111010_000011010000011),
                Gene(0b011_1001001_1111010_000010010010011),
                Gene(0b011_1001011_1111110_000011010000011),
            ]),
            //species 2
            Genome::from(vec![
                Gene(0b011_1001001_1011010_010011010000011),
                Gene(0b001_1001001_1011010_001010010010011),
                Gene(0b010_1001011_1111010_010011010000011),
            ]),
        ];

        let (species, clustered) = Species::cluster(&(genomes.iter().map(|g| g).collect()), 1e-1);
        assert_eq!(3, species.len());
        assert_eq!(vec![0, 0, 0, 1, 1, 2], clustered);
    }
}
