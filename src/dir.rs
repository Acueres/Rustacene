use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::coord::Coord;

#[derive(Copy, Clone, PartialEq)]
pub enum Dir {
    NULL,
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
        match rng.gen_range(0..9) {
            0 => Dir::NULL,
            1 => Dir::N,
            2 => Dir::S,
            3 => Dir::E,
            4 => Dir::W,
            5 => Dir::NE,
            6 => Dir::NW,
            7 => Dir::SE,
            _ => Dir::SW,
        }
    }
}

impl Dir {
    pub fn value(self) -> Coord<isize> {
        match self {
            Self::NULL => Coord { x: 0, y: 0 },
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
            0 => Self::NULL,
            1 => Self::N,
            2 => Self::S,
            3 => Self::E,
            4 => Self::W,
            5 => Self::NE,
            6 => Self::NW,
            7 => Self::SE,
            8 => Self::SW,
            _ => Self::NULL,
        }
    }
}
