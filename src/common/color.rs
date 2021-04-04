use std::fmt;

/// Predefined color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Color(r, g, b) = *self;
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl Into<(f32, f32, f32)> for Color {
    fn into(self) -> (f32, f32, f32) {
        let Color(r, g, b) = self;
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}

pub const GREEN: Color = Color(0, 255, 0);
pub const RED: Color = Color(255, 0, 0);
pub const BLUE: Color = Color(0, 0, 255);
pub const PURPLE: Color = Color(255, 0, 255);
pub const YELLOW: Color = Color(255, 255, 0);
pub const AQUA: Color = Color(0, 255, 255);
