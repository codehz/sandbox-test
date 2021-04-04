use std::ops::Range;

use crate::{
    math::axis::{Axis, ExtractAxis, HasAxisMut, HasAxisMutExt, MapAxisExt, SortAxisExt},
    math::trit::Trit,
};

use super::{
    chunk::{BlockSubPos, Chunk},
    ChunkPos, MapSize,
};

pub struct BlockIter {
    size: MapSize,
    position: glam::Vec3A,
    next_offset: glam::Vec3A,
    delta: glam::Vec3A,
    length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockIterResult {
    pub fine_position: glam::UVec3,
    pub direction: [Trit; 3],
    pub length: f32,
}

fn is_origin_inside(size: MapSize, position: glam::Vec3A) -> bool {
    let (x, y, z) = position.into();
    let (world_max_x, world_max_y, world_max_z) = size.to_world_size().into();
    (0.0..=world_max_x as f32).contains(&x)
        && (0.0..=world_max_y as f32).contains(&y)
        && (0.0..=world_max_z as f32).contains(&z)
}

fn is_position_inside(size: MapSize, position: glam::Vec3A) -> bool {
    let (x, y, z) = position.into();
    let (world_max_x, world_max_y, world_max_z) = size.to_world_size().into();
    (0.0..world_max_x as f32).contains(&x)
        && (0.0..world_max_y as f32).contains(&y)
        && (0.0..world_max_z as f32).contains(&z)
}

fn merge_range(origin: &mut Range<f32>, rhs: Range<f32>) -> bool {
    if rhs.end - origin.start > f32::EPSILON && origin.end - rhs.start > f32::EPSILON {
        *origin = origin.start.max(rhs.start)..origin.end.min(rhs.end);
        true
    } else {
        false
    }
}

fn gen_range(a: f32, b: f32) -> Range<f32> {
    a.min(b)..a.max(b)
}

fn get_origin_point(
    size: MapSize,
    position: glam::Vec3A,
    direction: glam::Vec3A,
) -> Option<(glam::Vec3A, f32)> {
    if is_origin_inside(size, position) {
        Some((position, 0.0))
    } else {
        let world_size = size.to_world_size();
        let mut range = f32::NEG_INFINITY..f32::INFINITY;
        for axis in direction.sort_axis(|axis, val| val > axis) {
            let base = position.extract_axis(axis);
            let invspeed = 1.0 / direction.extract_axis(axis);
            let bound = world_size.extract_axis(axis) as f32;
            let tmp = gen_range((-base) * invspeed, (bound - base) * invspeed);
            if !merge_range(&mut range, tmp) {
                return None;
            }
        }
        let tmin = range.start;
        if tmin > 0.0 {
            let new_position = position + direction * tmin;
            Some((new_position, tmin))
        } else {
            None
        }
    }
}

fn extract_fine_position(position: glam::Vec3A, is_negative: [bool; 3]) -> glam::UVec3 {
    Axis::generate(|axis| {
        let dir = is_negative.extract_axis(axis);
        let pos = position.extract_axis(axis);
        if dir && pos.fract() == 0.0 {
            (pos - 1.0) as u32
        } else {
            pos as u32
        }
    })
}

fn extract_position(fine_position: glam::UVec3) -> (ChunkPos, BlockSubPos) {
    use Axis::*;
    let fine_pos: [usize; 3] = fine_position.map_axis(|_, val| val as usize);
    let cx = (fine_pos.extract_axis(X) / Chunk::WIDTH) as u8;
    let cz = (fine_pos.extract_axis(Z) / Chunk::WIDTH) as u8;
    let bx = (fine_pos.extract_axis(X) % Chunk::WIDTH) as u8;
    let by = fine_pos.extract_axis(Y) as u8;
    let bz = (fine_pos.extract_axis(Z) % Chunk::WIDTH) as u8;
    (ChunkPos(cx, cz), BlockSubPos::new(bx, by, bz))
}

fn get_next_offset_delta(
    origin: glam::Vec3A,
    direction: glam::Vec3A,
) -> (glam::Vec3A, glam::Vec3A) {
    let delta = 1.0 / direction;
    let arr = Axis::generate_array(|axis| {
        let pos = origin.extract_axis(axis);
        let dir = direction.extract_axis(axis);
        let whole_time = delta.extract_axis(axis);
        let next_frame = if dir >= 0.0 {
            pos.ceil()
        } else if pos.fract() == 0.0 {
            pos - 1.0
        } else {
            pos.floor()
        };
        let next_time = (next_frame - pos) * whole_time;
        (next_time, whole_time)
    });
    (
        arr.map_axis(|_, (first, _)| first),
        arr.map_axis(|_, (_, second)| second),
    )
}

impl Default for BlockIter {
    fn default() -> Self {
        todo!()
    }
}

impl BlockIter {
    pub fn new(size: MapSize, start_position: glam::Vec3A, direction: glam::Vec3A) -> Self {
        let direction = match direction.try_normalize() {
            Some(direction) => direction,
            None => return Default::default(),
        };
        if let Some((origin_point, length)) = get_origin_point(size, start_position, direction) {
            let (next_offset, delta) = get_next_offset_delta(origin_point, direction);
            Self {
                size,
                position: extract_fine_position(
                    origin_point,
                    direction.map_axis(|_, val| val < 0.0),
                )
                .map_axis(|_, val| val as f32),
                next_offset,
                delta,
                length,
            }
        } else {
            Default::default()
        }
    }

    fn step(&mut self, time: f32) -> [Trit; 3] {
        self.length += time;
        Axis::generate_array(|axis| {
            let vel = self.delta.extract_axis(axis);
            if vel.is_infinite() {
                return Trit::Zero;
            }
            let sig = vel.signum();
            let vel = vel.abs();
            let origin = self.next_offset.extract_axis(axis);
            let next = origin - time;
            if next < f32::EPSILON {
                self.next_offset.set_axis(axis, vel);
                self.position.apply_axis(axis, |value| value + sig);
                (sig > 0.0).into()
            } else {
                self.next_offset.set_axis(axis, next);
                Default::default()
            }
        })
    }
}

impl Iterator for BlockIter {
    type Item = BlockIterResult;

    fn next(&mut self) -> Option<Self::Item> {
        let min_axis = self.next_offset.sort_axis(|a, b| a < b).next().unwrap();
        let direction = self.step(self.next_offset.extract_axis(min_axis));
        if is_position_inside(self.size, self.position) {
            Some(BlockIterResult {
                fine_position: self.position.map_axis(|_, val| val as u32),
                direction,
                length: self.length,
            })
        } else {
            None
        }
    }
}

impl BlockIterResult {
    pub fn get_position(self) -> (ChunkPos, BlockSubPos) {
        extract_position(self.fine_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_origin_point() {
        let size = MapSize::new((1, 1));
        let position = glam::vec3a(-1.0, -1.0, -1.0);
        let direction = glam::vec3a(1.0, 1.0, 1.0);
        let direction = direction.normalize();
        assert_eq!(
            get_origin_point(size, position, direction),
            Some((glam::vec3a(0.0, 0.0, 0.0), 1.7320509))
        );
        {
            let position = glam::vec3a(1.0, -1.0, 1.0);
            assert_eq!(
                get_origin_point(size, position, direction),
                Some((glam::vec3a(2.0, 0.0, 2.0), 1.7320509))
            );
        }
        {
            let position = glam::vec3a(1.0, 1.0, 1.0);
            assert_eq!(
                get_origin_point(size, position, direction),
                Some((glam::vec3a(1.0, 1.0, 1.0), 0.0))
            );
        }
        {
            let direction = glam::vec3a(1.0, -1.0, 1.0);
            let direction = direction.normalize();
            assert_eq!(get_origin_point(size, position, direction), None);
        }
        {
            let position = glam::vec3a(-1.0, -1.0, 0.0);
            let direction = glam::vec3a(1.0, 2.0, 1.0);
            let direction = direction.normalize();
            assert_eq!(
                get_origin_point(size, position, direction),
                Some((glam::vec3a(0.0, 1.0, 1.0), 2.4494898))
            );
        }
        {
            let world_size = size.to_world_size();
            let position = glam::vec3a(
                world_size.x as f32 + 0.0,
                world_size.y as f32 + 1.0,
                world_size.z as f32 + 0.0,
            );
            let direction = glam::vec3a(-1.0, -2.0, -1.0);
            let direction = direction.normalize();
            assert_eq!(
                get_origin_point(size, position, direction),
                Some((glam::vec3a(15.5, 64.0, 15.5), 1.2247449))
            );
        }
        {
            let position = glam::vec3a(1.0, 1024.0, 1.0);
            assert_eq!(get_origin_point(size, position, direction), None);
        }
    }

    #[test]
    fn test_get_next_offset_delta() {
        assert_eq!(
            get_next_offset_delta(glam::vec3a(0.5, 0.5, 0.5), glam::vec3a(1.0, 0.0, 0.0)),
            (
                glam::vec3a(0.5, f32::INFINITY, f32::INFINITY),
                glam::vec3a(1.0, f32::INFINITY, f32::INFINITY)
            )
        );
        assert_eq!(
            get_next_offset_delta(glam::vec3a(0.5, 0.5, 0.5), glam::vec3a(-1.0, 0.0, 0.0)),
            (
                glam::vec3a(0.5, f32::INFINITY, f32::INFINITY),
                glam::vec3a(-1.0, f32::INFINITY, f32::INFINITY)
            )
        );
        assert_eq!(
            get_next_offset_delta(glam::vec3a(0.5, 0.5, 0.5), glam::vec3a(1.0, 1.0, 2.0)),
            (glam::vec3a(0.5, 0.5, 0.25), glam::vec3a(1.0, 1.0, 0.5))
        );
    }

    #[test]
    fn test_iter() {
        let mut iter = BlockIter::new(
            MapSize::new((1, 1)),
            glam::vec3a(0.5, 0.5, 0.5),
            glam::vec3a(1.0, 0.0, 0.0),
        );
        assert_eq!(
            iter.next().unwrap().get_position(),
            (ChunkPos(0, 0), BlockSubPos::new(1, 0, 0))
        );
        assert_eq!(
            iter.next().unwrap().get_position(),
            (ChunkPos(0, 0), BlockSubPos::new(2, 0, 0))
        );
        assert_eq!(iter.next().unwrap().length, 2.5);

        let mut iter = BlockIter::new(
            MapSize::new((1, 1)),
            glam::vec3a(0.5, 0.5, 0.5),
            glam::vec3a(-1.0, 0.0, 0.0),
        );
        assert_eq!(iter.next(), None);

        let mut iter = BlockIter::new(
            MapSize::new((1, 1)),
            glam::vec3a(0.5, 0.5, 0.5),
            glam::vec3a(1.0, 1.0, 1.0),
        );
        assert_eq!(
            iter.next().unwrap().get_position(),
            (ChunkPos(0, 0), BlockSubPos::new(1, 1, 1))
        );

        let mut iter = BlockIter::new(
            MapSize::new((1, 1)),
            glam::vec3a(0.5, 0.5, 0.5),
            glam::vec3a(0.2, 0.0, 1.0),
        );
        assert_eq!(
            iter.next().unwrap().get_position(),
            (ChunkPos(0, 0), BlockSubPos::new(0, 0, 1))
        );
    }
}
