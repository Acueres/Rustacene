use crate::models::{Coord, Dir};
use bevy::prelude::Resource;
use ndarray::Array2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Empty,
    Impassable,
    Consumable,
}

#[derive(Resource, Clone)]
pub struct Grid {
    pub data: Array2<CellType>,
}

impl Grid {
    pub fn init(shape: (usize, usize)) -> Self {
        let v = vec![CellType::Empty; shape.0 * shape.1];
        Self {
            data: Array2::<CellType>::from_shape_vec(shape, v).unwrap(),
        }
    }

    pub fn search_along_dir(
        &self,
        x_origin: usize,
        y_origin: usize,
        distance: usize,
        dir: Dir,
        cell_type: CellType,
    ) -> Coord<isize> {
        let dir_coord = dir.value();
        let origin_coord = Coord::<isize>::new(x_origin as isize, y_origin as isize);
        let mut current_coord = origin_coord;

        let shape = self.data.shape();
        let x_range = 0..shape[0] as isize;
        let y_range = 0..shape[1] as isize;

        for _ in 0..distance {
            current_coord += dir_coord;
            if !(x_range.contains(&current_coord.x) && y_range.contains(&current_coord.y)) {
                break;
            }

            let current_cell_type = self.data[[current_coord.x as usize, current_coord.y as usize]];
            if current_cell_type == cell_type {
                return current_coord;
            }
        }

        origin_coord
    }

    pub fn _set(&mut self, x: usize, y: usize, cell_type: CellType) {
        self.data[[x, y]] = cell_type;
    }

    pub fn search_area(
        &self,
        origin: Coord<isize>,
        radius: usize,
        cell_type: CellType,
    ) -> Vec<Coord<isize>> {
        let grid_size = self.data.dim().0;
        let mut neighbors = Vec::<Coord<isize>>::new();

        let start = -(radius as isize);
        let end = radius as isize + 1;

        for x in start..end {
            for y in start..end {
                let neighbor = origin
                    + Coord::<isize> {
                        x: x as isize,
                        y: y as isize,
                    };
                if (x == 0 && y == 0)
                    || neighbor.x < 0
                    || neighbor.y < 0
                    || neighbor.x as usize >= grid_size
                    || neighbor.y as usize >= grid_size
                {
                    continue;
                }

                if self.data[[neighbor.x as usize, neighbor.y as usize]] == cell_type {
                    neighbors.push(neighbor);
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn test_search() {
        let mut grid = Grid::init((100, 100));

        let origin = Coord::<isize>::new(10, 10);
        let target_coord = Coord::<isize>::new(12, 12);
        grid._set(
            target_coord.x as usize,
            target_coord.y as usize,
            CellType::Consumable,
        );

        let test_coord = grid.search_along_dir(
            origin.x as usize,
            origin.y as usize,
            2,
            Dir::NE,
            CellType::Consumable,
        );
        assert_eq!(target_coord, test_coord);
    }

    #[test]
    fn test_search_boundary() {
        let grid = Grid::init((100, 100));

        let origin = Coord::<isize>::new(0, 0);

        let test_coord = grid.search_along_dir(
            origin.x as usize,
            origin.y as usize,
            5,
            Dir::NW,
            CellType::Empty,
        );
        assert_eq!(origin, test_coord);
    }

    #[test]
    fn test_search_area() {
        let grid = Grid::init((100, 100));
        let coord = Coord::<isize>::new(10, 10);
        let neighbors = grid.search_area(coord, 1, CellType::Empty);
        assert!(neighbors.len() == 8);

        let grid = Grid::init((100, 100));
        let coord = Coord::<isize>::new(0, 0);
        let neighbors = grid.search_area(coord, 1, CellType::Empty);
        assert!(neighbors.len() == 3);
    }
}
