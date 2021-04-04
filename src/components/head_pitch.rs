use std::{
    fmt,
    ops::{Add, AddAssign},
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct HeadPitch(pub f32);

impl fmt::Display for HeadPitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HeadPitch({})", self.0)
    }
}

impl Add<f32> for HeadPitch {
    type Output = HeadPitch;

    #[inline(always)]
    fn add(self, rhs: f32) -> Self::Output {
        const DEG_NEG_90: f32 = -std::f32::consts::FRAC_PI_2;
        const DEG_POS_90: f32 = std::f32::consts::FRAC_PI_2;
        Self((self.0 + rhs).clamp(DEG_NEG_90, DEG_POS_90))
    }
}

impl AddAssign<f32> for HeadPitch {
    #[inline(always)]
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}
