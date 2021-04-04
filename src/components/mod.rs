mod head_pitch;
mod model_structure;
mod position;
mod receive_gravity;
mod rotation;
mod sprite;
mod user_control;
mod velocity;

pub use head_pitch::HeadPitch;
pub use model_structure::ModelStructure;
pub use position::Position;
pub use receive_gravity::ReceiveGravity;
pub use rotation::Rotation;
pub use sprite::Sprite;
pub use user_control::UserControl;
pub use velocity::Velocity;

#[derive(Debug, Clone, Copy, bevy_ecs::Bundle)]
pub struct EntityBundle {
    pub position: Position,
    pub velocity: Velocity,
    pub rotation: Rotation,
    pub head_pitch: HeadPitch,
    pub model_structure: ModelStructure,
}
