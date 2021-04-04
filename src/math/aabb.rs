use std::ops::{BitXor, Range};

use strum::IntoEnumIterator;

use super::axis::{Axis, ExtractAxis, HasAxisMut, MapAxisExt};

#[derive(Debug, Default, Clone, Copy)]
pub struct AABB {
    pub position: glam::Vec3A,
    pub extent3d: glam::Vec3A,
}

impl std::fmt::Display for AABB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { position, extent3d } = self;
        write!(f, "AABB({}, {})", position, extent3d)
    }
}

impl ExtractAxis for AABB {
    type Target = Range<f32>;

    fn extract_axis(&self, axis: Axis) -> Self::Target {
        let min = self.min();
        let max = self.max();
        min.extract_axis(axis)..max.extract_axis(axis)
    }
}

impl From<[Range<f32>; 3]> for AABB {
    fn from([x, y, z]: [Range<f32>; 3]) -> Self {
        Self {
            position: glam::vec3a(x.start, y.start, z.start),
            extent3d: glam::vec3a(x.end - x.start, y.end - y.start, z.end - z.start),
        }
    }
}

impl AABB {
    #[inline(always)]
    pub fn valid(self) -> bool {
        let arr: [bool; 3] = self.extent3d.map_axis(|_, value| value.is_sign_positive());
        arr.iter().all(|x| *x)
    }

    pub fn min(self) -> glam::Vec3A {
        self.position
    }
    pub fn max(self) -> glam::Vec3A {
        self.position + self.extent3d
    }

    pub fn center(self) -> glam::Vec3A {
        let Self { position, extent3d } = self;
        position + glam::vec3a(extent3d.x / 2.0, extent3d.y / 2.0, extent3d.x / 2.0)
    }

    pub fn split_y(self) -> (Self, Self) {
        let Self { position, extent3d } = self;
        let extent3d = extent3d * glam::vec3a(1.0, 0.5, 1.0);
        let high_position = position + glam::vec3a(0.0, extent3d.y, 0.0);
        (
            Self { position, extent3d },
            Self {
                position: high_position,
                extent3d,
            },
        )
    }

    pub fn expanded(self, value: glam::Vec3A) -> Self {
        let half = value / glam::vec3a(2.0, 2.0, 2.0);
        let Self { position, extent3d } = self;
        Self {
            position: position - half,
            extent3d: extent3d + value,
        }
    }

    pub fn split_by_axis(self, axis: Axis, is_pos: bool) -> Self {
        let Self { position, extent3d } = self;
        let extent3d = extent3d * {
            let mut ret = glam::vec3a(1.0, 1.0, 1.0);
            ret.set_axis(axis, 0.5f32);
            ret
        };
        if is_pos {
            let high_position = position + {
                let mut ret = glam::Vec3A::ZERO;
                ret.set_axis(axis, extent3d.extract_axis(axis));
                ret
            };
            Self {
                position: high_position,
                extent3d,
            }
        } else {
            Self { position, extent3d }
        }
    }

    pub fn from_block_pos(position: glam::Vec3A) -> Self {
        Self {
            position,
            extent3d: glam::const_vec3a!([1.0, 1.0, 1.0]),
        }
    }

    pub fn from_ab(a: glam::Vec3A, b: glam::Vec3A) -> Self {
        let (a, b) = (a.min(b), a.max(b));
        Self {
            position: a,
            extent3d: b - a,
        }
    }

    pub fn areas(self) -> glam::Vec3A {
        self.map_axis(|axis, _| {
            axis.rest()
                .iter()
                .map(move |&axis| self.extract_axis(axis))
                .map(|rng| (rng.end - rng.start).abs())
                .fold(1.0, |a, b| a * b)
        })
    }

    pub fn max_axis(self) -> Axis {
        let mut max_area = 0.0f32;
        let mut ret = Axis::X;
        for axis in Axis::iter() {
            let area = axis
                .rest()
                .iter()
                .map(move |&axis| self.extract_axis(axis))
                .map(|rng| (rng.end - rng.start).abs())
                .fold(1.0, |a, b| a * b);
            if area > max_area {
                max_area = area;
                ret = axis;
            }
        }
        ret
    }

    pub fn iter_pos(self) -> impl Iterator<Item = glam::UVec3> {
        let Self { position, extent3d } = self;
        let (minx, miny, minz) = position.floor().max(glam::Vec3A::default()).into();
        let (maxx, maxy, maxz) = (position + extent3d).ceil().into();
        let (minx, miny, minz) = (minx as u32, miny as u32, minz as u32);
        let (maxx, maxy, maxz) = (maxx as u32, maxy as u32, maxz as u32);
        itertools::iproduct!(minx..maxx, miny..maxy, minz..maxz)
            .map(|(x, y, z)| glam::uvec3(x, y, z))
    }
}

impl BitXor for AABB {
    type Output = AABB;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.map_axis(|axis, value| {
            let rhs = rhs.extract_axis(axis);
            value.start.max(rhs.start)..value.end.min(rhs.end)
        })
    }
}
