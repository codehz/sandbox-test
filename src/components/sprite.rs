use crate::{
    common::color::Color,
    math::aabb::{IntoAABB, AABB},
};

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub color: Color,
    pub radius: f32,
}

impl Sprite {
    pub fn new(color: Color, radius: f32) -> Self {
        Self { color, radius }
    }
}

impl IntoAABB for Sprite {
    fn into_aabb<T>(&self, position: T) -> AABB
    where
        T: Into<glam::Vec3A>,
    {
        let origin: glam::Vec3A = position.into();
        let radius = self.radius;
        let diff = glam::Vec3A::splat(radius);
        AABB {
            position: origin - diff,
            extent3d: diff * 2.0,
        }
    }
}
