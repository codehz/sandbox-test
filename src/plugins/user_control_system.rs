use bevy_app::{stage, EventReader, Events, Plugin};
use bevy_ecs::{IntoSystem, Local, Query, Res, ResMut, State, StateStage};

use crate::{
    components::{HeadPitch, ModelStructure, Position, Rotation, UserControl},
    renderer::{camera::Camera, events::*, Action},
    resources::{ControlConfig, KeyboardTracing, PickedBlock},
    world::{
        block::{Block, BlockType},
        block_iter::BlockIter,
        Map,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserInputState {
    Disabled,
    Enabled,
}

pub static USER_INPUT_STATE: &str = "user input";

fn init_user_input(mut keyboard_tracing: ResMut<KeyboardTracing>) {
    use VirtualKeyCode::*;
    keyboard_tracing.add_key(Space);
    keyboard_tracing.add_key(W);
    keyboard_tracing.add_key(A);
    keyboard_tracing.add_key(S);
    keyboard_tracing.add_key(D);
}

fn tracing_keyboard_system(
    mut keyboard_tracing: ResMut<KeyboardTracing>,
    keyboard_events: Res<Events<KeyboardInput>>,
    mut keyboard_event_reader: Local<EventReader<KeyboardInput>>,
) {
    keyboard_event_reader
        .iter(&keyboard_events)
        .filter_map(|input| input.virtual_keycode.map(|key| (key, input.state)))
        .for_each(|(key, state)| keyboard_tracing.set(key, state));
}

fn user_input_system(
    control_config: Res<ControlConfig>,
    mouse_motion_events: Res<Events<MouseMotionEvent>>,
    mut mouse_motion_event_reader: Local<EventReader<MouseMotionEvent>>,
    keyboard_tracing: Res<KeyboardTracing>,
    mut query: Query<&mut UserControl>,
) {
    use ElementState::*;
    use VirtualKeyCode as Key;
    let mut uc = match query.iter_mut().last() {
        Some(it) => it,
        _ => return,
    };
    for &MouseMotionEvent(x, y) in mouse_motion_event_reader.iter(&mouse_motion_events) {
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

fn state_transition<const ENTER: bool>(mut action: ResMut<Events<Action>>) {
    action.send(Action::CaptureMouse(ENTER));
}

fn handle_in_game(
    keyboard_events: Res<Events<KeyboardInput>>,
    mut keyboard_event_reader: Local<EventReader<KeyboardInput>>,
    focused_events: Res<Events<FocusedEvent>>,
    mut focused_event_reader: Local<EventReader<FocusedEvent>>,
    mut ui_state: ResMut<State<UserInputState>>,
) {
    use glium::glutin::event::*;
    for event in keyboard_event_reader.iter(&keyboard_events) {
        if let &KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Escape),
            ..
        } = event
        {
            ui_state.set_next(UserInputState::Disabled).unwrap();
            return;
        }
    }
    for event in focused_event_reader.iter(&focused_events) {
        if !event.0 {
            ui_state.set_next(UserInputState::Disabled).unwrap();
            return;
        }
    }
}

fn handle_paused_game(
    mouse_button_events: Res<Events<MouseButtonEvent>>,
    mut mouse_button_event_reader: Local<EventReader<MouseButtonEvent>>,
    keyboard_events: Res<Events<KeyboardInput>>,
    mut keyboard_event_reader: Local<EventReader<KeyboardInput>>,
    mut exit: ResMut<Events<Action>>,
    mut app_state: ResMut<State<UserInputState>>,
) {
    use glium::glutin::event::*;
    for event in mouse_button_event_reader.iter(&mouse_button_events) {
        if let &MouseButtonEvent {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        } = event
        {
            app_state.set_next(UserInputState::Enabled).unwrap();
            break;
        }
    }
    for event in keyboard_event_reader.iter(&keyboard_events) {
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
    mouse_button_events: Res<Events<MouseButtonEvent>>,
    mut mouse_button_event_reader: Local<EventReader<MouseButtonEvent>>,
) {
    use glium::glutin::event::*;
    let picked = match *picked {
        Some(picked) => picked,
        None => return,
    };
    for event in mouse_button_event_reader.iter(&mouse_button_events) {
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

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, appb: &mut bevy_app::AppBuilder) {
        let mut stat = StateStage::<UserInputState>::default();
        stat.on_state_update(UserInputState::Disabled, handle_paused_game.system())
            .on_state_enter(UserInputState::Enabled, state_transition::<true>.system())
            .on_state_update(UserInputState::Enabled, picking_system.system())
            .on_state_update(UserInputState::Enabled, handle_in_game.system())
            .on_state_update(UserInputState::Enabled, break_block_system.system())
            .on_state_update(UserInputState::Enabled, tracing_keyboard_system.system())
            .on_state_update(UserInputState::Enabled, user_input_system.system())
            .on_state_exit(UserInputState::Enabled, reset_user_input_system.system())
            .on_state_exit(UserInputState::Enabled, state_transition::<false>.system());
        appb.add_resource(KeyboardTracing::default())
            .add_resource::<Option<PickedBlock>>(None)
            .add_resource(State::new(UserInputState::Disabled))
            .add_system(player_camera_system.system())
            .add_startup_system(init_user_input.system())
            .add_stage_before(stage::UPDATE, USER_INPUT_STATE, stat);
    }
}
