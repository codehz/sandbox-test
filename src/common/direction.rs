use enum_map::Enum;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, EnumIter)]
pub enum Direction {
    North, // Z-
    South, // Z+
    East, // X+
    West, // X-
    Up, // Y+
    Down, // Y-
}
