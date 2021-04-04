mod keyboard_tracing;
mod picked_block;

pub use keyboard_tracing::*;
pub use picked_block::*;

#[derive(Debug, Clone, Copy)]
pub struct ControlConfig {
    pub rotation_scale: glam::Vec2,
}

