use crate::components::{CellType, Coord, Dir};
use crate::resources::*;

#[inline]
pub fn process_sensors(
    coord: &Coord<isize>,
    grid: &Grid,
    params: &Parameters,
    dir: Dir,
) -> Vec<f32> {
    let x_coord = coord.x as f32 / params.grid_size as f32;
    let y_coord = coord.y as f32 / params.grid_size as f32;
    let x_coord_inv = 1. - x_coord;
    let y_coord_inv = 1. - y_coord;

    let pellet_coord = grid.search_along_dir(
        coord.x as usize,
        coord.y as usize,
        5,
        dir,
        CellType::Consumable,
    );

    let pellet_exists = if *coord != pellet_coord { 1. } else { 0. };

    vec![x_coord, y_coord, x_coord_inv, y_coord_inv, pellet_exists]
}
