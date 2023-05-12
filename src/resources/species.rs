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
