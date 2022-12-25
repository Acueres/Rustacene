use crate::coord::Coord;
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

    pub fn get_coords(
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
    use crate::coord::Coord;

    #[test]
    fn test_free_coords() {
        let grid = Grid::init((100, 100));
        let coord = Coord::<isize> { x: 10, y: 10 };
        let neighbors = grid.get_coords(coord, 1, CellType::Empty);
        assert!(neighbors.len() == 8);

        let grid = Grid::init((100, 100));
        let coord = Coord::<isize> { x: 0, y: 0 };
        let neighbors = grid.get_coords(coord, 1, CellType::Empty);
        assert!(neighbors.len() == 3);
    }
}
