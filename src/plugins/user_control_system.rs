use bevy_app::{EventReader, Events, Plugin};
use bevy_ecs::prelude::*;

use crate::{
    common::color,
    components::{HeadPitch, ModelStructure, Position, Rotation, Sprite, UserControl, Velocity},
    renderer::{camera::Camera, events::*, Action},
    resources::{ControlConfig, KeyboardTracing, PickedBlock},
    world::{
        block::{Block, BlockType},
        block_iter::BlockIter,
        Map,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserInputState {
    Disabled,
    Enabled,
}

fn init_user_input(mut commands: Commands) {
    use VirtualKeyCode::*;
    let mut tracing = KeyboardTracing::default();
    tracing.add_key(Space);
    tracing.add_key(W);
    tracing.add_key(A);
    tracing.add_key(S);
    tracing.add_key(D);
    commands.insert_resource(tracing);
}

fn tracing_keyboard_system(
    mut keyboard_tracing: ResMut<KeyboardTracing>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
) {
    keyboard_event_reader
        .iter()
        .filter_map(|input| input.virtual_keycode.map(|key| (key, input.state)))
        .for_each(|(key, state)| keyboard_tracing.set(key, state));
}

fn user_input_system(
    control_config: Res<ControlConfig>,
    mut mouse_motion_event_reader: EventReader<MouseMotionEvent>,
    keyboard_tracing: Res<KeyboardTracing>,
    mut query: Query<&mut UserControl>,
) {
    use ElementState::*;
    use VirtualKeyCode as Key;
    let mut uc = match query.iter_mut().last() {
        Some(it) => it,
        _ => return,
    };
    for &MouseMotionEvent(x, y) in mouse_motion_event_reader.iter() {
        let scale = control_config.rotation_scale;
        uc.rotation += glam::vec2(x, y) * scale;
    }
    uc.jumping = keyboard_tracing[Key::Space] == Pressed;
    uc.moving = glam::Vec2::default();
    for &key in &[Key::W, Key::A, Key::S, Key::D] {
        let state = keyboard_tracing[key];
        let tmp = match key {
            Key::W => glam::vec2(0.0, (state == Pressed) as u32 as f32),
            Key::S => glam::vec2(0.0, -((state == Pressed) as u32 as f32)),
            Key::D => glam::vec2((state == Pressed) as u32 as f32, 0.0),
            Key::A => glam::vec2(-((state == Pressed) as u32 as f32), 0.0),
            _ => panic!("invalid branch"),
        };
        uc.moving += tmp;
    }
}

fn reset_user_input_system(
    mut picked: ResMut<Option<PickedBlock>>,
    mut keyboard_tracing: ResMut<KeyboardTracing>,
    mut uc_query: Query<&mut UserControl>,
) {
    picked.take();
    keyboard_tracing.reset();
    let mut uc = match uc_query.iter_mut().last() {
        Some(it) => it,
        _ => return,
    };
    *uc = UserControl::default();
}

fn player_camera_system(
    query: Query<(
        &Position,
        &Rotation,
        &HeadPitch,
        &UserControl,
        &ModelStructure,
    )>,
    mut camera: ResMut<Camera>,
) {
    if let Some((pos, rot, pitch, uc, structure)) = query.iter().last() {
        camera.eye = pos.0 + glam::vec3a(0.0, structure.head_offset, 0.0);
        camera.yaw = rot.0 + uc.rotation.x;
        camera.pitch = pitch.0 + uc.rotation.y;
    }
}

fn mouse_capture<const ENTER: bool>(mut action: ResMut<Events<Action>>) {
    action.send(Action::CaptureMouse(ENTER));
}

fn handle_in_game(
    mut keyboard_event_reader: EventReader<KeyboardInput>,
    mut focused_event_reader: EventReader<FocusedEvent>,
    mut ui_state: ResMut<State<UserInputState>>,
) {
    use glium::glutin::event::*;
    for event in keyboard_event_reader.iter() {
        if let &KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Escape),
            ..
        } = event
        {
            ui_state.set(UserInputState::Disabled).unwrap();
            return;
        }
    }
    for event in focused_event_reader.iter() {
        if !event.0 {
            ui_state.set(UserInputState::Disabled).unwrap();
            return;
        }
    }
}

fn handle_paused_game(
    mut mouse_button_event_reader: EventReader<MouseButtonEvent>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
    mut exit: ResMut<Events<Action>>,
    mut app_state: ResMut<State<UserInputState>>,
) {
    use glium::glutin::event::*;
    for event in mouse_button_event_reader.iter() {
        if let &MouseButtonEvent {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        } = event
        {
            app_state.set(UserInputState::Enabled).unwrap();
            break;
        }
    }
    for event in keyboard_event_reader.iter() {
        if let &KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Escape),
            ..
        } = event
        {
            exit.send(Action::Exit);
            return;
        }
    }
}

fn break_block_system(
    picked: Res<Option<PickedBlock>>,
    mut map: ResMut<Map>,
    mut mouse_button_event_reader: EventReader<MouseButtonEvent>,
) {
    use glium::glutin::event::*;
    let picked = match *picked {
        Some(picked) => picked,
        None => return,
    };
    for event in mouse_button_event_reader.iter() {
        if event.state == ElementState::Released {
            continue;
        }
        match event.button {
            MouseButton::Left => {
                let (chunk_pos, block_sub_pos) = map.size().convert_pos(picked.position).unwrap();
                log::info!(
                    "breaking {} ({} {})",
                    picked.position,
                    chunk_pos,
                    block_sub_pos
                );
                map[chunk_pos][block_sub_pos].take();
            }
            MouseButton::Right => {
                if let Some((chunk_pos, block_sub_pos)) = map
                    .size()
                    .convert_pos_with_offset(picked.position, picked.direction.into())
                {
                    map[chunk_pos][block_sub_pos]
                        .replace(crate::world::block::constants::YELLOW_BLOCK);
                }
            }
            _ => {}
        }
    }
}

fn picking_system(mut picked: ResMut<Option<PickedBlock>>, map: Res<Map>, camera: Res<Camera>) {
    let position = camera.eye;
    let direction = camera.get_direction();
    let size = map.size();

    *picked = BlockIter::new(size, position, direction)
        .take_while(|reslut| reslut.length <= 8.0)
        .find_map(|result| {
            let (chunk_pos, block_pos) = result.get_position();
            map[chunk_pos][block_pos].map(|blk| {
                match blk {
                    Block {
                        data: BlockType::Solid { .. },
                        ..
                    } => {}
                }
                PickedBlock {
                    position: result.fine_position,
                    direction: result.direction,
                }
            })
        });
}

fn generate_sprite_system(
    mut mouse_button_event_reader: EventReader<MouseButtonEvent>,
    mut commands: Commands,
    camera: Res<Camera>,
) {
    for event in mouse_button_event_reader.iter() {
        if event.state == ElementState::Released {
            continue;
        }
        match event.button {
            MouseButton::Middle => {
                let dir = camera.get_direction();
                let pos = camera.eye;
                commands.spawn_bundle((
                    Sprite {
                        color: color::RED,
                        radius: 0.1,
                    },
                    Position(pos),
                    Velocity(dir * 0.2),
                ));
            }
            _ => {}
        }
    }
}

pub struct UserInputPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum UserInputLabel {
    CameraSystem,
    StateSwitcher,
    ResetState,
    PickingSystem,
    GameState,
    PlayerAction,
    KeyboardTracing,
    UpdateUserControl,
}

impl Plugin for UserInputPlugin {
    fn build(&self, appb: &mut bevy_app::AppBuilder) {
        appb.insert_resource(Option::<PickedBlock>::None)
            .add_state(UserInputState::Disabled)
            .add_startup_system(init_user_input.system())
            .add_system(
                player_camera_system
                    .system()
                    .label(UserInputLabel::CameraSystem),
            )
            .add_system_set(
                SystemSet::on_update(UserInputState::Disabled)
                    .with_system(handle_paused_game.system().label(UserInputLabel::GameState)),
            )
            .add_system_set(
                SystemSet::on_enter(UserInputState::Enabled)
                    .with_system(mouse_capture::<true>.system())
                    .label(UserInputLabel::StateSwitcher),
            )
            .add_system_set(
                SystemSet::on_update(UserInputState::Enabled)
                    .with_system(
                        tracing_keyboard_system
                            .system()
                            .label(UserInputLabel::KeyboardTracing),
                    )
                    .with_system(picking_system.system().label(UserInputLabel::PickingSystem))
                    .with_system(handle_in_game.system().label(UserInputLabel::GameState))
                    .with_system(
                        generate_sprite_system
                            .system()
                            .label(UserInputLabel::PlayerAction),
                    )
                    .with_system(
                        break_block_system
                            .system()
                            .label(UserInputLabel::PlayerAction),
                    )
                    .with_system(
                        user_input_system
                            .system()
                            .label(UserInputLabel::UpdateUserControl),
                    ),
            )
            .add_system_set(
                SystemSet::on_exit(UserInputState::Enabled)
                    .with_system(
                        reset_user_input_system
                            .system()
                            .label(UserInputLabel::ResetState)
                            .after(UserInputLabel::KeyboardTracing),
                    )
                    .with_system(
                        mouse_capture::<false>
                            .system()
                            .label(UserInputLabel::StateSwitcher),
                    ),
            );
    }
}
