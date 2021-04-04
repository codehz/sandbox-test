use crate::{common::direction::Direction};

#[derive(Debug, Clone, Copy)]
pub struct PickedBlock {
    pub position: glam::UVec3,
    pub direction: Direction,
}
