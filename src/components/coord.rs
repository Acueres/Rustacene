use bevy::prelude::Component;
use std::ops::{Add, AddAssign, Sub};

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct Coord<T> {
    pub x: T,
    pub y: T,
}

impl<T> Coord<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Sub<Output = T>> Sub for Coord<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Coord {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Add<Output = T>> Add for Coord<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Coord {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: AddAssign> AddAssign for Coord<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
