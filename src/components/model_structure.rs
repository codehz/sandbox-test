use crate::math::aabb::AABB;

#[derive(Debug, Clone, Copy)]
pub struct ModelStructure {
    pub width: f32,
    pub height: f32,
    pub head_offset: f32,
}

impl ModelStructure {
    pub fn get_aabb<T>(&self, position: T) -> AABB
    where
        T: Into<glam::Vec3A>,
    {
        let &Self { width, height, .. } = self;
        AABB {
            position: position.into() - glam::vec3a(width / 2.0, 0.0, width / 2.0),
            extent3d: glam::vec3a(width, height, width),
        }
    }

    pub fn get_extent(&self) -> glam::Vec2 {
        let &Self { width, height, .. } = self;
        glam::vec2(width, height)
    }
}
