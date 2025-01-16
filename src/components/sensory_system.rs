use crate::components::{CellType, Coord, Dir};
use crate::resources::*;
use bevy::prelude::{Component, Vec2};
use ndarray::{Array1, Array2};

#[derive(Component, Clone)]
pub struct SensorySystem {
    weights: Array1<f32>,
    fov_angle: f32,
}

impl SensorySystem {
    pub const SENSOR_RANGE: usize = 5;
    pub const N_SENSORS: usize = Self::SENSOR_RANGE * 2 + 1;
    const FOV_ANGLE: f32 = 46.;

    pub fn new(weights: Vec<f32>) -> Self {
        Self {
            weights: Array1::from_vec(weights),
            fov_angle: Self::FOV_ANGLE.to_radians(),
        }
    }

    pub fn process_data(&self, grid: &Grid, origin: Coord<isize>, dir: Dir) -> Vec<f32> {
        let cell_data = grid.get_area(origin, Self::SENSOR_RANGE);
        let dim = cell_data.dim();
        let mut sensor_data = Array2::<f32>::zeros(dim);
        let dir_coord = dir.value();

        for x in 0..dim.0 {
            for y in 0..dim.1 {
                let cell_type = cell_data[[x, y]];
                if cell_type == CellType::Empty {
                    continue;
                }

                let coord = Coord::<isize>::new(
                    x as isize - Self::SENSOR_RANGE as isize,
                    Self::SENSOR_RANGE as isize - y as isize,
                );

                let angle = Vec2::new(coord.x as f32, coord.y as f32)
                    .angle_to(Vec2::new(dir_coord.x as f32, dir_coord.y as f32));

                if angle.abs() > self.fov_angle {
                    continue;
                }

                sensor_data[[x, y]] = if cell_type == CellType::Consumable {
                    1.
                } else {
                    -1.
                };
            }
        }

        sensor_data
            .t()
            .dot(&self.weights)
            .into_iter()
            .map(|v| v.tanh())
            .collect()
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn test_sensory_system() {
        let mut grid = Grid::new((100, 100));
        let origin = Coord::<isize>::new(50, 50);

        //test north quadrant
        grid.set(46, 45, CellType::Impassable);
        grid.set(50, 45, CellType::Consumable);

        //test south quadrant
        grid.set(45, 55, CellType::Consumable);
        grid.set(50, 55, CellType::Consumable);
        grid.set(50, 53, CellType::Impassable);
        grid.set(55, 55, CellType::Consumable);

        //test east quadrant
        grid.set(55, 54, CellType::Impassable);

        //test values are precalculated manually
        let ss = SensorySystem::new(vec![0.6, 0.3, 0., 0.8, 1.2, -0.1, 0.5, -1.3, 2.1, 0., 1.4]);

        let res = ss.process_data(&grid, origin, Dir::N);
        assert_eq!(11, res.len());
        assert_eq!(((-0.4_f32).tanh() * 1e6) as usize, (res[0] * 1e6) as usize);

        let res = ss.process_data(&grid, origin, Dir::S);
        assert_eq!((0.1_f32.tanh() * 1e6) as usize, (res[8] * 1e6) as usize);
        assert_eq!((1.9_f32.tanh() * 1e6) as usize, (res[10] * 1e6) as usize);

        let res = ss.process_data(&grid, origin, Dir::E);
        assert_eq!(((-1.4_f32).tanh() * 1e6) as usize, (res[9] * 1e6) as usize);
        assert_eq!((1.4_f32.tanh() * 1e6) as usize, (res[10] * 1e6) as usize);
    }
}
