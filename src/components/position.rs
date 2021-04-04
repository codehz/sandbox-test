use std::{
    fmt,
    ops::{Add, AddAssign},
};

use super::Velocity;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Position(pub glam::Vec3A);

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.into();
        write!(f, "Position({}, {}, {})", x, y, z)
    }
}

impl Into<(f32, f32, f32)> for Position {
    fn into(self) -> (f32, f32, f32) {
        (self.0.x, self.0.y, self.0.z)
    }
}

impl Into<(f32, f32, f32)> for &Position {
    fn into(self) -> (f32, f32, f32) {
        (self.0.x, self.0.y, self.0.z)
    }
}

impl From<(f32, f32, f32)> for Position {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self(glam::vec3a(x, y, z))
    }
}

impl From<[f32; 3]> for Position {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self(glam::vec3a(x, y, z))
    }
}

impl Add<Velocity> for Position {
    type Output = Position;

    fn add(self, rhs: Velocity) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl AddAssign<Velocity> for Position {
    fn add_assign(&mut self, rhs: Velocity) {
        *self = *self + rhs;
    }
}
