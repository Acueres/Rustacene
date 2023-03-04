use super::Dir;
use rand::Rng;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Halt,
    MoveContinue,
    MoveRandom,
    MoveReverse,
    MoveN,
    MoveS,
    MoveE,
    MoveW,
    MoveNE,
    MoveNW,
    MoveSE,
    MoveSW,
}

impl Action {
    pub fn get_dir(self, prev_dir: Dir) -> Dir {
        match self {
            Self::MoveN => Dir::N,
            Self::MoveS => Dir::S,
            Self::MoveE => Dir::E,
            Self::MoveW => Dir::W,
            Self::MoveNE => Dir::NE,
            Self::MoveNW => Dir::NW,
            Self::MoveSE => Dir::SE,
            Self::MoveSW => Dir::SW,
            Self::MoveContinue => prev_dir,
            Self::MoveReverse => -prev_dir,
            Self::MoveRandom => rand::thread_rng().gen(),
            _ => panic!("Unknown value: {:?}", self),
        }
    }

    pub fn get(index: usize) -> Self {
        match index {
            0 => Self::Halt,
            1 => Self::MoveContinue,
            2 => Self::MoveRandom,
            3 => Self::MoveReverse,
            4 => Self::MoveN,
            5 => Self::MoveS,
            6 => Self::MoveE,
            7 => Self::MoveW,
            8 => Self::MoveNE,
            9 => Self::MoveNW,
            10 => Self::MoveSE,
            11 => Self::MoveSW,
            _ => panic!("Unknown value: {}", index),
        }
    }
}
