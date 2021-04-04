use std::fmt;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Velocity(pub glam::Vec3A);

impl fmt::Display for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.into();
        write!(f, "Position({}, {}, {})", x, y, z)
    }
}

impl Into<(f32, f32, f32)> for Velocity {
    fn into(self) -> (f32, f32, f32) {
        (self.0.x, self.0.y, self.0.z)
    }
}

impl Into<(f32, f32, f32)> for &Velocity {
    fn into(self) -> (f32, f32, f32) {
        (self.0.x, self.0.y, self.0.z)
    }
}

impl From<(f32, f32, f32)> for Velocity {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self(glam::vec3a(x, y, z))
    }
}

impl From<[f32; 3]> for Velocity {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self(glam::vec3a(x, y, z))
    }
}
