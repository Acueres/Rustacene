use crate::coord::Coord;
use crate::grid::{CellType, Grid};
use crate::models::Parameters;
use crate::organism::Organism;
use rand::Rng;

pub fn init_world(params: Parameters) -> (Vec<Organism>, Vec<Coord<isize>>, Grid) {
    let mut orgs = Vec::<Organism>::new();
    orgs.reserve_exact(params.n_initial_entities * 3);

    let mut coords = Vec::<Coord<isize>>::new();
    coords.reserve_exact(params.n_initial_entities * 3);

    let mut grid = Grid::init((params.grid_size, params.grid_size));

    let mut rng = rand::thread_rng();

    let mut n = 0;
    while n < params.n_initial_entities {
        let x = rng.gen_range(0..params.grid_size);
        let y = rng.gen_range(0..params.grid_size);

        if grid.data[[x, y]] == CellType::Impassable {
            continue;
        }

        grid.data[[x, y]] = CellType::Impassable;

        orgs.push(Organism::new(0.5, params.genome_len, params.ns_shape));

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        coords.push(coord);

        n += 1;
    }
    (orgs, coords, grid)
}
