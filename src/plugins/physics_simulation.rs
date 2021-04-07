use std::ops::Add;

use bevy_app::{CoreStage, Plugin};
use bevy_core::{Time, Timer};
use bevy_ecs::{prelude::*, schedule::ShouldRun};

use crate::{
    components::{
        HeadPitch, ModelStructure, Position, ReceiveGravity, Rotation, Sprite, UserControl,
        Velocity,
    },
    math::{
        aabb::{IntoAABB, AABB},
        axis::{ExtractAxis, HasAxis, HasAxisMut, MapAxisExt, SortAxisExt},
        bound3d::{Bound3D, LimitRange},
        voxel_bound::VoxelBound,
    },
    world::Map,
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct PhysicsPosition(glam::Vec3A);

impl Add<Velocity> for PhysicsPosition {
    type Output = glam::Vec3A;

    fn add(self, rhs: Velocity) -> Self::Output {
        self.0 + rhs.0
    }
}

impl From<Position> for PhysicsPosition {
    fn from(Position(value): Position) -> Self {
        Self(value)
    }
}

impl From<[f32; 3]> for PhysicsPosition {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self(glam::vec3a(x, y, z))
    }
}

impl HasAxis for PhysicsPosition {
    type Target = f32;

    fn get_axis(&self, axis: crate::math::axis::Axis) -> &Self::Target {
        self.0.get_axis(axis)
    }
}

impl HasAxisMut for PhysicsPosition {
    fn get_axis_mut(&mut self, axis: crate::math::axis::Axis) -> &mut Self::Target {
        self.0.get_axis_mut(axis)
    }
}

struct PhysicsTimer(Timer);

fn physics_tick_system(time: Res<Time>, mut timer: ResMut<PhysicsTimer>) -> ShouldRun {
    if timer.0.tick(time.delta()).just_finished() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn sync_position_system(
    map: Res<Map>,
    timer: Res<PhysicsTimer>,
    mut has_physics_position: Query<(&mut Position, &Bound3D, &Velocity, &PhysicsPosition)>,
    no_physics_position: Query<(Entity, &Position), Without<PhysicsPosition>>,
    mut commands: Commands,
) {
    let percent = timer.0.percent();
    for (mut pos, bound, vel, phys) in has_physics_position.iter_mut() {
        *pos = bound.apply(phys.0 + vel.0 * percent)
    }
    for (entity, &pos) in no_physics_position.iter() {
        commands
            .entity(entity)
            .insert_bundle((PhysicsPosition::from(pos), Bound3D::from_world(&map.size())));
    }
}

fn player_velocity_system(
    mut query: Query<(
        &mut Velocity,
        &mut Rotation,
        &mut HeadPitch,
        &mut UserControl,
    )>,
) {
    for (mut vel, mut rot, mut pitch, mut ctrl) in query.iter_mut() {
        let crot = ctrl.rotation;
        ctrl.rotation = glam::vec2(0.0, 0.0);
        *rot += crot.x;
        *pitch += crot.y;
        let (x, z) = ctrl.moving.into();
        let mut target_vel = rot.matrix().transform_point3(glam::vec3(x, 0.0, -z) * 0.2);
        target_vel.y = if ctrl.jumping {
            (vel.0.y + 0.02).max(0.1)
        } else {
            vel.0.y
        };
        // target_vel.y = vel.0.y + (if ctrl.jumping { 0.02 } else { 0.0 });
        *vel = Velocity(target_vel.into());
    }
}

fn gravity_system(mut query: Query<&mut Velocity, With<ReceiveGravity>>) {
    for mut vel in query.iter_mut() {
        vel.0 += glam::vec3a(0.0, -0.01, 0.0);
    }
}

fn sprite_collision_system(
    map: Res<Map>,
    mut query: Query<(Entity, &mut PhysicsPosition, &Velocity, &Sprite)>,
    mut commands: Commands,
) {
    let map_bound = Bound3D::from_world(&map.size());
    'outer: for (entity, mut pos, vel, &sprite) in query.iter_mut() {
        let next_pos = pos.0 + vel.0;
        if map_bound.out_of_bound(next_pos) {
            commands.entity(entity).despawn();
            continue 'outer;
        }
        let aabb = sprite.into_aabb(next_pos);
        for (_, blk) in map.scan_aabb(aabb) {
            match blk.data {
                crate::world::block::BlockType::Solid { .. } => {}
            }
            commands.entity(entity).despawn();
            continue 'outer;
        }
        pos.0 = next_pos;
    }
}

fn map_collision_detection(
    map: Res<Map>,
    mut query: Query<(
        &mut PhysicsPosition,
        &mut Velocity,
        &mut Bound3D,
        &ModelStructure,
    )>,
) {
    let map_bound = Bound3D::from_world(&map.size());
    for (mut pos, mut vel, mut cached_bound, &structure) in query.iter_mut() {
        let extent = structure.get_extent();
        *cached_bound = Default::default();
        for axis in vel.sort_axis(|a, b| a.abs() > b.abs()) {
            let vel_axis = vel.extract_axis(axis);
            let next_pos: glam::Vec3A = pos.adjust_axis(axis, |val| val + vel_axis);
            let aabb = structure
                .into_aabb(next_pos)
                .expanded(glam::vec3a(0.01, 0.0, 0.01));
            let mut voxel_bound = VoxelBound::default();
            for (target, blk) in map.scan_aabb(aabb) {
                match blk.data {
                    crate::world::block::BlockType::Solid { .. } => {}
                }
                let overlapped = AABB::from_block_pos(target) ^ aabb;
                let tmp = VoxelBound::from_aabb_axis(overlapped, axis);
                voxel_bound.merge(tmp);
            }
            let bound = {
                let mut ret = map_bound;
                ret.limit(axis, Into::<std::ops::Range<_>>::into(voxel_bound));
                ret.shrink_by(extent)
            };
            *cached_bound &= bound;
            *pos = bound.apply(next_pos);
            if pos.extract_axis(axis) != next_pos.extract_axis(axis) {
                vel.set_axis(axis, 0.0f32);
            }
        }
    }
}

static PHYSICS_SIMULATION: &str = "physics simulation";

pub struct PhysicsPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PhysicsLabel {
    SyncPosition,
    Gravity,
    Collision,
    PlayerVelocity,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, appb: &mut bevy_app::AppBuilder) {
        let mut stage = SystemStage::parallel().with_run_criteria(physics_tick_system.system());
        stage
            .add_system(gravity_system.system().label(PhysicsLabel::Gravity))
            .add_system(
                sprite_collision_system
                    .system()
                    .label(PhysicsLabel::Collision)
                    .after(PhysicsLabel::Gravity),
            )
            .add_system(
                map_collision_detection
                    .system()
                    .label(PhysicsLabel::Collision)
                    .after(PhysicsLabel::Gravity),
            )
            .add_system(
                player_velocity_system
                    .system()
                    .label(PhysicsLabel::PlayerVelocity)
                    .after(PhysicsLabel::Gravity)
                    .after(PhysicsLabel::Collision)
                    .after(PhysicsLabel::SyncPosition),
            );
        appb.insert_resource(PhysicsTimer(Timer::from_seconds(0.025, true)))
            .add_stage_after(CoreStage::Update, PHYSICS_SIMULATION, stage)
            .add_system(
                sync_position_system
                    .system()
                    .label(PhysicsLabel::SyncPosition),
            );
    }
}
