use num_traits::Zero;
use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellType {
    Empty,
    Impassable,
    Consumable,
}

impl Zero for CellType {
    fn zero() -> Self {
        Self::Empty
    }

    fn is_zero(&self) -> bool {
        *self == Self::Empty
    }

    fn set_zero(&mut self) {
        *self = Self::Empty;
    }
}

impl Add for CellType {
    type Output = Self;

    fn add(self, _: Self) -> Self {
        panic!("Operation not defined");
    }
}
