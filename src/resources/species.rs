use crate::components::Genome;
use bevy::prelude::Color;
use bevy::prelude::Resource;
use rand::Rng;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Resource, Clone)]
pub struct Species {
    species: HashSet<usize>,
    population_count: HashMap<usize, usize>,
    species_count: usize,
    colors: HashSet<(u8, u8, u8)>,
    color_map: HashMap<usize, (u8, u8, u8)>,
}

impl Species {
    pub fn new(species: HashSet<usize>) -> Self {
        let mut rng = rand::rng();
        let mut colors = HashSet::<(u8, u8, u8)>::with_capacity(species.len());
        let species_count = species.len();

        let population_count = HashMap::<usize, usize>::from_iter(species.iter().map(|s| (*s, 0)));

        while colors.len() < species_count {
            colors.insert((
                rng.random_range(0..u8::MAX),
                rng.random_range(0..u8::MAX),
                rng.random_range(0..u8::MAX),
            ));
        }

        let mut color_map = HashMap::<usize, (u8, u8, u8)>::with_capacity(species_count);
        for (s, c) in species.iter().zip(colors.iter()) {
            color_map.insert(*s, *c);
        }

        Self {
            species,
            population_count,
            species_count,
            colors,
            color_map,
        }
    }

    pub fn from_genomes(genomes: &Vec<&Genome>, genetic_threshold: f32) -> (Self, Vec<usize>) {
        let mut assigned_species = vec![0; genomes.len()];
        let mut unassigned = HashSet::<usize>::from_iter(0..genomes.len());
        let mut species = HashSet::<usize>::new();
        let mut species_count = 0;

        for (i, genome) in genomes.iter().enumerate() {
            if !unassigned.remove(&i) {
                continue;
            }

            assigned_species[i] = species_count;

            let species_members: Vec<_> = unassigned
                .iter()
                .map(|index| (*index, genome.get_distance(genomes[*index])))
                .filter(|(_, d)| *d < genetic_threshold)
                .map(|(index, _)| index)
                .collect();

            for index in species_members.iter() {
                assigned_species[*index] = species_count;
            }

            unassigned =
                &unassigned - &HashSet::<usize>::from_iter(species_members.iter().cloned());

            species.insert(species_count);
            species_count += 1;
        }

        let mut res = Self::new(species);
        for s in assigned_species.iter() {
            res.increment_species(*s);
        }

        (res, assigned_species)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.species.len()
    }

    #[inline]
    pub fn topk(&self, k: usize) -> Vec<(usize, usize)> {
        //reverse key and value so that elements would be ordered by value
        let mut heap = BinaryHeap::<(usize, usize)>::from_iter(
            self.population_count.iter().map(|(k, v)| (*v, *k)),
        );

        (0..k)
            .map(|_| {
                let pair = heap.pop().unwrap_or_default();
                return (pair.1, pair.0);
            })
            .collect()
    }

    #[inline]
    pub fn add_species(&mut self) -> usize {
        let species = self.species_count + 1;

        if self.species.insert(species) {
            self.species_count += 1;
            self.population_count.insert(species, 0);

            let mut rng = rand::rng();
            loop {
                let color = (
                    rng.random_range(0..u8::MAX),
                    rng.random_range(0..u8::MAX),
                    rng.random_range(0..u8::MAX),
                );
                if self.colors.insert(color) {
                    self.color_map.insert(species, color);
                    break;
                }
            }
        }

        species
    }

    #[inline]
    pub fn remove_species(&mut self, species: &usize) {
        self.species.remove(species);
        self.population_count.remove(species);

        let color = *self.color_map.get(species).unwrap();
        self.colors.remove(&color);
        self.color_map.remove(species);
    }

    #[inline]
    pub fn increment_species(&mut self, species: usize) {
        if let Some(n) = self.population_count.get_mut(&species) {
            *n += 1;
        }
    }

    #[inline]
    pub fn decrement_species(&mut self, species: usize) {
        if let Some(n) = self.population_count.get_mut(&species) {
            *n = if *n > 0 { *n - 1 } else { 0 };
            if *n == 0 {
                self.remove_species(&species);
            }
        }
    }

    #[inline]
    pub fn get_color(&self, species: usize) -> Color {
        if let Some(color) = self.color_map.get(&species) {
            return Color::srgb_u8(color.0, color.1, color.2);
        }

        Color::WHITE
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

        let (species, clustered) =
            Species::from_genomes(&(genomes.iter().map(|g| g).collect()), 1e-1);
        assert_eq!(3, species.len());
        assert_eq!(vec![0, 0, 0, 1, 1, 2], clustered);
    }

    #[test]
    fn test_top_species() {
        let mut species = Species::new(HashSet::<usize>::from_iter(0..10));

        for _ in 0..100 {
            species.increment_species(8);
        }

        for _ in 0..60 {
            species.increment_species(5);
        }

        for _ in 0..10 {
            species.increment_species(3);
        }

        for _ in 0..5 {
            species.increment_species(7);
        }

        let top_species = species.topk(4);

        assert_eq!(vec![(8, 100), (5, 60), (3, 10), (7, 5)], top_species);
    }
}
