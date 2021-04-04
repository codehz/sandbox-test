use std::{
    fmt,
    ops::{Add, AddAssign},
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rotation(pub f32);

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rotation({})", self.0)
    }
}

impl Rotation {
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_y(-self.0)
    }
}

impl Add<f32> for Rotation {
    type Output = Rotation;

    #[inline(always)]
    fn add(self, rhs: f32) -> Self::Output {
        const DEG_360: f32 = std::f32::consts::TAU;
        Self((self.0 + rhs).rem_euclid(DEG_360))
    }
}

impl AddAssign<f32> for Rotation {
    #[inline(always)]
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}
