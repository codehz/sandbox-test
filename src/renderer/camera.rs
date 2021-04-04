use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: glam::Vec3A,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub hard_range: Range<f32>,
    pub soft_range: Range<f32>,
}

impl Camera {
    #[inline(always)]
    pub fn view_model(&self) -> glam::Mat4 {
        let &Self {
            eye, yaw, pitch, ..
        } = self;
        let translation = glam::Mat4::from_translation((-eye).into());
        let rot_y = glam::Mat4::from_rotation_y(yaw);
        let rot_x = glam::Mat4::from_rotation_x(pitch);
        rot_x * rot_y * translation
    }

    #[inline(always)]
    pub fn perspective(&self, aspect_ratio: f32) -> glam::Mat4 {
        let Self {
            fov, hard_range, ..
        } = self;
        glam::Mat4::perspective_rh_gl(*fov, aspect_ratio, hard_range.start, hard_range.end)
    }

    pub fn get_direction(&self) -> glam::Vec3A {
        let &Self { yaw, pitch, .. } = self;
        let rot_y = glam::Mat3::from_rotation_y(-yaw);
        let rot_x = glam::Mat3::from_rotation_x(-pitch);
        let rot = rot_y * rot_x;
        rot * glam::vec3a(0.0, 0.0, -1.0)
    }
}
