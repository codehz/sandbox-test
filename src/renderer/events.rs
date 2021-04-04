pub use glium::glutin::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

#[derive(Debug)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub state: ElementState,
}

impl MouseButtonEvent {
    pub fn new(button: MouseButton, state: ElementState) -> Self {
        Self { button, state }
    }
}

#[derive(Debug)]
pub struct MouseMotionEvent(pub f32, pub f32);

#[derive(Debug)]
pub struct FocusedEvent(pub bool);
