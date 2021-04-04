#[derive(Debug, Default, Clone, Copy)]
pub struct UserControl {
    pub moving: glam::Vec2,
    pub rotation: glam::Vec2,
    pub jumping: bool,
}
