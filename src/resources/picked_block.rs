use crate::{common::direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PickedBlock {
    pub position: glam::UVec3,
    pub direction: Direction,
}
