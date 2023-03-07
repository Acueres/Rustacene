use crate::components::Coord;
use crate::resources::Grid;
use crate::components::CellType;
use rand::seq::SliceRandom;

pub fn generate_pellets(n_entities: usize, grid: &Grid) -> Vec<Coord<isize>> {
    let rng = &mut rand::thread_rng();
    let n_pellets = (100 * (250 / n_entities)).clamp(0, n_entities);

    grid.data
        .indexed_iter()
        .filter(|x| x.1.to_owned() == CellType::Empty)
        .collect::<Vec<((usize, usize), &CellType)>>()
        .iter()
        .map(|v| Coord::<isize> {
            x: v.0 .0 as isize,
            y: v.0 .1 as isize,
        })
        .collect::<Vec<Coord<isize>>>()
        .choose_multiple(rng, n_pellets)
        .cloned()
        .collect::<Vec<Coord<isize>>>()
}
