use crate::components::{CellType, Coord, Organism, SensorySystem};
use crate::resources::{Grid, Parameters, Species};
use rand::Rng;

const INITIAL_ENERGY: f32 = 0.2;

pub fn init_world(params: Parameters) -> (Vec<Organism>, Species, Vec<Coord<isize>>, Grid) {
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

        orgs.push(Organism::new(
            INITIAL_ENERGY,
            params.n_initial_connections + params.n_initial_neurons + SensorySystem::N_SENSORS,
        ));

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    let (species, assigned_species) =
        Species::from_genomes(&(orgs.iter().map(|o| &o.genome).collect()), 1e-1);
    orgs.iter_mut()
        .zip(assigned_species.iter())
        .for_each(|(org, species)| org.species = *species);

    (orgs, species, coords, grid)
}
