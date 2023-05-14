use crate::components::{CellType, Coord, Genome, Organism};
use crate::resources::{Grid, Parameters};

use rand::Rng;
use std::collections::HashSet;

const INITIAL_ENERGY: f32 = 0.2;

pub fn init_world(params: Parameters) -> (Vec<Organism>, HashSet<usize>, Vec<Coord<isize>>, Grid) {
    let mut orgs = Vec::<Organism>::with_capacity(params.n_initial_entities * 3);
    let mut coords = Vec::<Coord<isize>>::with_capacity(params.n_initial_entities * 3);

    let mut grid = Grid::new((params.grid_size, params.grid_size));

    let mut rng = rand::thread_rng();

    let mut n = 0;
    while n < params.n_initial_entities {
        let x = rng.gen_range(0..params.grid_size);
        let y = rng.gen_range(0..params.grid_size);

        if grid.get(x, y) == CellType::Impassable {
            continue;
        }

        grid.set(x, y, CellType::Impassable);

        orgs.push(Organism::new(INITIAL_ENERGY, params.genome_len));

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    let (species_data, species) =
        cluster_species(&(orgs.iter().map(|o| &o.genome).collect()), 1e-1);
    orgs.iter_mut()
        .zip(species_data.iter())
        .for_each(|(org, species)| org.species = *species);

    (orgs, species, coords, grid)
}

fn cluster_species(genomes: &Vec<&Genome>, threshold: f32) -> (Vec<usize>, HashSet<usize>) {
    let mut species_data = vec![0; genomes.len()];
    let mut unassigned = HashSet::<usize>::from_iter(0..genomes.len());
    let mut species = HashSet::<usize>::new();
    let mut species_counter = 0;

    for (i, genome) in genomes.iter().enumerate() {
        if !unassigned.remove(&i) {
            continue;
        }

        species_data[i] = species_counter;

        let species_members: Vec<_> = unassigned
            .iter()
            .map(|index| (*index, genome.get_distance(genomes[*index])))
            .filter(|(_, d)| *d < threshold)
            .map(|(index, _)| index)
            .collect();

        for index in species_members.iter() {
            species_data[*index] = species_counter;
        }

        unassigned = &unassigned - &HashSet::<usize>::from_iter(species_members.iter().cloned());

        species.insert(species_counter);
        species_counter += 1;
    }

    (species_data, species)
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

        let (clustered, species) = cluster_species(&(genomes.iter().map(|g| g).collect()), 1e-1);
        assert_eq!(3, species.len());
        assert_eq!(vec![0, 0, 0, 1, 1, 2], clustered);
    }
}
