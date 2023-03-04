use crate::coord::Coord;
use crate::dir::Dir;
use crate::resources::*;
use crate::resources::{CellType, Grid};

#[inline]
pub fn process_sensors(
    coord: &Coord<isize>,
    grid: &Grid,
    params: &Parameters,
    dir: Dir,
) -> Vec<f32> {
    let x_coord = coord.x as f32 / params.grid_size as f32;
    let y_coord = coord.y as f32 / params.grid_size as f32;

    let pellet_coord = grid.search_along_dir(
        coord.x as usize,
        coord.y as usize,
        3,
        dir,
        CellType::Consumable,
    );
    let x_pellet = 1. - (x_coord - (pellet_coord.x as f32 / params.grid_size as f32));
    let y_pellet = 1. - (y_coord - (pellet_coord.y as f32 / params.grid_size as f32));

    vec![x_coord, y_coord, x_pellet, y_pellet]
}
