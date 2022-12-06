use crate::coord::Coord;
use bevy::prelude::Resource;
use ndarray::Array2;

#[derive(Resource, Clone)]
pub struct Grid {
    pub data: Array2<bool>,
}

impl Grid {
    pub fn init(shape: (usize, usize)) -> Self {
        let v = vec![false; shape.0 * shape.1];
        Self {
            data: Array2::<bool>::from_shape_vec(shape, v).unwrap(),
        }
    }

    pub fn get_free_coords(self, coord: Coord<isize>) -> Vec<Coord<isize>> {
        let grid_size = self.data.dim().0;
        let mut neighbors = Vec::<Coord<isize>>::new();
        for x in -1..2 {
            for y in -1..2 {
                let neighbor = coord
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

                if !self.data[[neighbor.x as usize, neighbor.y as usize]] {
                    neighbors.push(neighbor);
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod grid_tests {
    use super::Grid;
    use crate::coord::Coord;

    #[test]
    fn test_free_coords() {
        let grid = Grid::init((100, 100));
        let coord = Coord::<isize> { x: 10, y: 10 };
        let neighbors = grid.get_free_coords(coord);
        assert!(neighbors.len() == 8);

        let grid = Grid::init((100, 100));
        let coord = Coord::<isize> { x: 0, y: 0 };
        let neighbors = grid.get_free_coords(coord);
        assert!(neighbors.len() == 3);
    }
}
