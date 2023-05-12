use crate::components::{cluster_species, CellType, Coord, Organism};
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
