use super::Coord;
use bevy::prelude::Component;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::ops::Neg;

#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub enum Dir {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir {
        match rng.gen_range(0..8) {
            0 => Dir::N,
            1 => Dir::S,
            2 => Dir::E,
            3 => Dir::W,
            4 => Dir::NE,
            5 => Dir::NW,
            6 => Dir::SE,
            7 => Dir::SW,
            _ => panic!(),
        }
    }
}

impl Dir {
    pub fn value(self) -> Coord<isize> {
        match self {
            Self::N => Coord { x: 0, y: 1 },
            Self::S => Coord { x: 0, y: -1 },
            Self::E => Coord { x: 1, y: 0 },
            Self::W => Coord { x: -1, y: 0 },
            Self::NE => Coord { x: 1, y: 1 },
            Self::NW => Coord { x: -1, y: 1 },
            Self::SE => Coord { x: 1, y: -1 },
            Self::SW => Coord { x: -1, y: -1 },
        }
    }

    pub fn get(index: usize) -> Self {
        match index {
            0 => Self::N,
            1 => Self::S,
            2 => Self::E,
            3 => Self::W,
            4 => Self::NE,
            5 => Self::NW,
            6 => Self::SE,
            7 => Self::SW,
            _ => panic!("Unknown value: {}", index),
        }
    }

    //*Clockwise rotation */
    pub fn rotate(self) -> Self {
        match self {
            Self::N => Self::NE,
            Self::S => Self::SW,
            Self::E => Self::SE,
            Self::W => Self::NW,
            Self::NE => Self::E,
            Self::NW => Self::N,
            Self::SE => Self::S,
            Self::SW => Self::W,
        }
    }

    //*Counterclockwise rotation */
    pub fn rotate_counter(self) -> Self {
        match self {
            Self::N => Self::NW,
            Self::S => Self::SE,
            Self::E => Self::NE,
            Self::W => Self::SW,
            Self::NE => Self::N,
            Self::NW => Self::W,
            Self::SE => Self::E,
            Self::SW => Self::S,
        }
    }

    pub fn to_arr(self) -> [f32; 4] {
        let mut res = [0.; 4];

        match self {
            Self::N => {
                res[0] = 1.;
            }
            Self::S => {
                res[1] = 1.;
            }
            Self::E => {
                res[2] = 1.;
            }
            Self::W => {
                res[3] = 1.;
            }
            Self::NE => {
                res[0] = 1.;
                res[2] = 1.;
            }
            Self::NW => {
                res[0] = 1.;
                res[3] = 1.;
            }
            Self::SE => {
                res[1] = 1.;
                res[2] = 1.;
            }
            Self::SW => {
                res[1] = 1.;
                res[3] = 1.;
            }
        }

        res
    }
}

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::N => Self::S,
            Self::S => Self::N,
            Self::E => Self::W,
            Self::W => Self::E,
            Self::NE => Self::SW,
            Self::SW => Self::NE,
            Self::NW => Self::SE,
            Self::SE => Self::NW,
        }
    }
}
