use crate::components::*;
use crate::resources::Grid;
use rand::seq::SliceRandom;

const MAX_ENERGY: f32 = 100.;

pub fn energy_system(total_org_energy: f32, grid: &Grid) -> Vec<Coord<isize>> {
    let rng = &mut rand::thread_rng();
    let total_pellet_energy =
        grid.get_cell_coords(CellType::Consumable).iter().len() as f32 * PELLET_ENERGY;
    let total_energy = total_org_energy + total_pellet_energy;
    let n_pellets = (((MAX_ENERGY - total_energy) * 0.05) / PELLET_ENERGY) as usize;
    if n_pellets <= 0 {
        return Vec::<Coord<isize>>::new();
    }

    grid.get_cell_coords(CellType::Empty)
        .choose_multiple(rng, n_pellets)
        .cloned()
        .collect::<Vec<Coord<isize>>>()
}
