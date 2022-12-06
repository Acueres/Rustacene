use bevy::prelude::Component;
use std::ops::{Add, Sub};

#[derive(Component, Clone, Copy, PartialEq)]
pub struct Coord<T> {
    pub x: T,
    pub y: T,
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
