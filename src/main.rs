use bevy_app::{App, Plugin};
use bevy_core::CorePlugin;
use bevy_ecs::prelude::*;
use sandbox_test as lib;

use lib::{
    components::{EntityBundle, ModelStructure, ReceiveGravity, UserControl},
    pipeline, plugins,
    renderer::{self, pass::*, RenderPlugin},
};

fn startup(mut commands: Commands) {
    commands
        .spawn_bundle(EntityBundle {
            position: (5.0, 70.0, 5.0).into(),
            velocity: (0.0, 0.0, 0.0).into(),
            rotation: Default::default(),
            head_pitch: Default::default(),
            model_structure: ModelStructure {
                width: 0.8,
                height: 1.5,
                head_offset: 1.2,
            },
        })
        .insert_bundle((ReceiveGravity, UserControl::default()));
}
struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, appb: &mut bevy_app::AppBuilder) {
        let control = lib::resources::ControlConfig {
            rotation_scale: glam::vec2(0.01, 0.005),
        };
        let camera = renderer::camera::Camera {
            eye: glam::vec3a(15.0, 5.0, 15.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 60.0f32.to_radians(),
            hard_range: 0.1..64.0,
            soft_range: 0.0..64.0,
        };
        let map = lib::world::Map::new(
            (8, 8),
            lib::world::generator::noise::NoiseGenerator::new(
                noise::ScalePoint::new(noise::HybridMulti::new()).set_scale(0.03),
                noise::ScalePoint::new(noise::Worley::new()).set_scale(0.07),
            ),
        );
        appb.insert_resource(camera)
            .insert_resource(control)
            .insert_resource(map)
            .add_startup_system(startup.system());
    }
}

pub fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    App::build()
        .insert_resource(bevy_ecs::schedule::ReportExecutionOrderAmbiguities)
        .add_plugin(CorePlugin)
        .add_plugin(GamePlugin)
        .add_plugin(plugins::PhysicsPlugin)
        .add_plugin(plugins::UserInputPlugin)
        .add_plugin(RenderPlugin::<
            pipeline!(
                cube::CubePass,
                sprite::SpritePass,
                // debug::DebugPass,
                outline::OutlinePass,
                strengthen::StrengthenPass
            ),
        >::default())
        .run();
}
