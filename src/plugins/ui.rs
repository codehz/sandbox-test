use bevy_app::{AppBuilder, Plugin};
use bevy_ecs::prelude::*;

use crate::renderer::pass::ui::{Texture, UiConcept};

pub struct UiPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiState {
    Empty,
}

impl Plugin for UiPlugin {
    fn build(&self, builder: &mut AppBuilder) {
        builder
            .add_state(UiState::Empty)
            .insert_non_send_resource(imgui::Context::create())
            .insert_non_send_resource(imgui::Textures::<Texture>::new())
            .insert_resource(Box::new(|world: &mut World, ui: &mut imgui::Ui| {
                use imgui::*;
                let [w, h] = ui.io().display_size;
                Window::new(im_str!("Measurement"))
                    .size([w, h], Condition::Always)
                    .flags(
                        WindowFlags::NO_DECORATION
                            | WindowFlags::NO_MOVE
                            | WindowFlags::NO_BACKGROUND,
                    )
                    .position([0.0, 0.0], Condition::Always)
                    .position_pivot([0.0, 0.0])
                    .build(ui, || {
                        use bevy_diagnostic::*;
                        let diag = world.get_resource::<Diagnostics>().unwrap();
                        if let Some(fps) =
                            diag.get(FrameTimeDiagnosticsPlugin::FPS).unwrap().value()
                        {
                            ui.text(ImString::new(format!("fps: {}", fps)));
                        }
                        if let Some(entity_count) = diag
                            .get(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                            .unwrap()
                            .value()
                        {
                            ui.text(ImString::new(format!("entity count: {}", entity_count)));
                        }
                    })
            }) as Box<dyn UiConcept>);
    }
}
