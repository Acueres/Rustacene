use super::Dir;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    Halt,
    MoveContinue,
    MoveRandom,
    MoveReverse,
    Rotate,
    RotateCounter,
}

impl Action {
    pub const N_ACTIONS: usize = 6;

    pub fn get_dir(self, curr_dir: Dir) -> Dir {
        match self {
            Self::MoveContinue => curr_dir,
            Self::MoveRandom => rand::random(),
            Self::MoveReverse => -curr_dir,
            Self::Rotate => curr_dir.rotate(),
            Self::RotateCounter => curr_dir.rotate_counter(),
            _ => panic!("Action doesn't exist: {:?}", self),
        }
    }

    pub fn get(index: usize) -> Self {
        match index {
            0 => Self::Halt,
            1 => Self::MoveContinue,
            2 => Self::MoveRandom,
            3 => Self::MoveReverse,
            4 => Self::Rotate,
            5 => Self::RotateCounter,
            _ => panic!("Action doesn't exist: {}", index),
        }
    }
}
