use crate::components::CellType;
use crate::components::Coord;
use crate::resources::Grid;
use rand::seq::SliceRandom;

pub fn generate_pellets(n_entities: usize, grid: &Grid) -> Vec<Coord<isize>> {
    let rng = &mut rand::thread_rng();
    let n_pellets = (100 * (250 / n_entities)).clamp(0, n_entities);

    grid.get_cell_coords(CellType::Empty)
        .choose_multiple(rng, n_pellets)
        .cloned()
        .collect::<Vec<Coord<isize>>>()
}
