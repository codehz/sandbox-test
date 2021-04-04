use std::ops::Index;

use glium::glutin::event::{ElementState, VirtualKeyCode};

#[derive(Default, Debug, Clone)]
pub struct KeyboardTracing(std::collections::BTreeMap<VirtualKeyCode, ElementState>);

impl KeyboardTracing {
    pub fn add_key(&mut self, key: VirtualKeyCode) {
        self.0.insert(key, ElementState::Released);
    }

    pub fn set(&mut self, key: VirtualKeyCode, state: ElementState) {
        self.0.entry(key).and_modify(|x| *x = state);
    }

    pub fn reset(&mut self) {
        for (_, value) in &mut self.0 {
            *value = ElementState::Released;
        }
    }
}

impl Index<VirtualKeyCode> for KeyboardTracing {
    type Output = ElementState;

    fn index(&self, index: VirtualKeyCode) -> &Self::Output {
        &self.0[&index]
    }
}
