use crate::math::trit::Trit;

#[derive(Debug, Default, Clone, Copy)]
pub struct PickedBlock {
    pub position: glam::UVec3,
    pub direction: [Trit; 3],
}
