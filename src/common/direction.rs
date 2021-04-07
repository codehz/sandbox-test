use enum_map::Enum;
use strum_macros::EnumIter;

use crate::math::axis::Axis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord, Enum, EnumIter)]
pub enum Direction {
    North, // Z-
    South, // Z+
    East,  // X+
    West,  // X-
    Up,    // Y+
    Down,  // Y-
}

impl Direction {
    pub fn from_axis(axis: Axis, is_positive: bool) -> Self {
        use Direction::*;
        let arr = match axis {
            Axis::X => [West, East],
            Axis::Y => [Down, Up],
            Axis::Z => [North, South],
        };
        arr[is_positive as usize]
    }
}

impl Into<glam::IVec3> for Direction {
    fn into(self) -> glam::IVec3 {
        match self {
            Direction::North => glam::ivec3(0, 0, 1),
            Direction::South => glam::ivec3(0, 0, -1),
            Direction::East => glam::ivec3(-1, 0, 0),
            Direction::West => glam::ivec3(1, 0, 0),
            Direction::Up => glam::ivec3(0, -1, 0),
            Direction::Down => glam::ivec3(0, 1, 0),
        }
    }
}
