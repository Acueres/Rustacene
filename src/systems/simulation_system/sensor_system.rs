use crate::components::{CellType, Coord, Dir, Organism};
use crate::resources::*;

pub const N_SENSORS: usize = 10;

#[inline]
pub fn read_sensors(
    org: &Organism,
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
        10,
        dir,
        CellType::Consumable,
    );

    let pellet_exists = if *coord != pellet_coord { 1. } else { 0. };
    let dir_arr = dir.to_arr();

    vec![
        x_coord,
        y_coord,
        x_coord_inv,
        y_coord_inv,
        dir_arr[0], // north component
        dir_arr[1], // south component
        dir_arr[2], // east component
        dir_arr[3], // west component
        pellet_exists,
        org.energy,
    ]
}
