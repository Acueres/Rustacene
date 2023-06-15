use crate::components::{CellType, Coord};
use bevy::prelude::Resource;
use ndarray::Array2;

#[derive(Resource, Clone)]
pub struct Grid {
    data: Array2<CellType>,
}

impl Grid {
    pub fn new(shape: (usize, usize)) -> Self {
        let v = vec![CellType::Empty; shape.0 * shape.1];
        Self {
            data: Array2::<CellType>::from_shape_vec(shape, v).unwrap(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell_type: CellType) {
        self.data[[x, y]] = cell_type;
    }

    pub fn get(&self, x: usize, y: usize) -> CellType {
        self.data[[x, y]]
    }

    pub fn get_cell_coords(&self, cell_type: CellType) -> Vec<Coord<isize>> {
        self.data
            .indexed_iter()
            .filter(|x| x.1.to_owned() == cell_type)
            .collect::<Vec<((usize, usize), &CellType)>>()
            .iter()
            .map(|v| Coord::<isize> {
                x: v.0 .0 as isize,
                y: v.0 .1 as isize,
            })
            .collect::<Vec<Coord<isize>>>()
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
                let coord = origin
                    + Coord::<isize> {
                        x: x as isize,
                        y: y as isize,
                    };
                if (x == 0 && y == 0)
                    || coord.x < 0
                    || coord.y < 0
                    || coord.x as usize >= grid_size
                    || coord.y as usize >= grid_size
                {
                    continue;
                }

                if self.data[[coord.x as usize, coord.y as usize]] == cell_type {
                    neighbors.push(coord);
                }
            }
        }

        neighbors
    }

    pub fn get_area(&self, origin: Coord<isize>, radius: usize) -> Array2<CellType> {
        let grid_size = self.data.dim().0;
        let len = radius * 2 + 1;
        let mut res = Array2::<CellType>::zeros((len, len));

        let start = -(radius as isize);
        let end = radius as isize + 1;

        for x in start..end {
            for y in start..end {
                let coord = origin
                    + Coord::<isize> {
                        x: x as isize,
                        y: y as isize,
                    };
                if coord.x < 0
                    || coord.y < 0
                    || coord.x as usize >= grid_size
                    || coord.y as usize >= grid_size
                {
                    continue;
                }

                res[[
                    (x + radius as isize) as usize,
                    (y + radius as isize) as usize,
                ]] = self.data[[coord.x as usize, coord.y as usize]];
            }
        }

        res
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn test_search_area() {
        let grid = Grid::new((100, 100));
        let origin = Coord::<isize>::new(10, 10);
        let neighbors = grid.search_area(origin, 1, CellType::Empty);
        assert!(neighbors.len() == 8);

        let grid = Grid::new((100, 100));
        let origin = Coord::<isize>::new(0, 0);
        let neighbors = grid.search_area(origin, 1, CellType::Empty);
        assert!(neighbors.len() == 3);
    }

    #[test]
    fn test_get_area() {
        let mut grid = Grid::new((100, 100));
        let origin = Coord::<isize>::new(50, 50);

        grid.set(50, 55, CellType::Consumable);
        grid.set(55, 55, CellType::Consumable);
        grid.set(46, 45, CellType::Impassable);

        let area = grid.get_area(origin, 5);

        assert_eq!(11 * 11, area.len());
        assert_eq!(CellType::Consumable, area[[5, 10]]);
        assert_eq!(CellType::Consumable, area[[10, 10]]);
        assert_eq!(CellType::Impassable, area[[1, 0]]);
        assert_eq!(CellType::Empty, area[[3, 2]]);
    }
}
