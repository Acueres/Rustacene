use crate::coord::Coord;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::ops::Neg;

#[derive(Copy, Clone, PartialEq, Debug)]
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
            _ => panic!("Range error"),
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
