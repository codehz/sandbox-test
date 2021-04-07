use std::ops::{BitAnd, BitAndAssign, Range, RangeFrom, RangeFull, RangeTo};

use strum::IntoEnumIterator;

use crate::world::{chunk::Chunk, MapSize};

use super::axis::{Axis, ExtractAxis, HasAxis, HasAxisMutExt, MapAxisExt};

#[derive(Debug, Clone, Copy)]
enum LimitMode {
    NoLimit,
    LimitMin,
    LimitMax,
}

impl Default for LimitMode {
    fn default() -> Self {
        Self::NoLimit
    }
}

impl LimitMode {
    fn from_diff(center: glam::Vec3A, target: glam::Vec3A) -> [Self; 3] {
        let diff = target + glam::vec3a(0.5, 0.5, 0.5) - center;
        diff.map_axis(|_, val| {
            if val < 0.0 {
                Self::LimitMax
            } else if val > 0.0 {
                Self::LimitMin
            } else {
                Self::NoLimit
            }
        })
    }

    fn calc_block_bound(self, value: f32) -> Range<f32> {
        match self {
            LimitMode::NoLimit => f32::NEG_INFINITY..f32::INFINITY,
            LimitMode::LimitMin => f32::NEG_INFINITY..value,
            LimitMode::LimitMax => (value + 1.0)..f32::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bound3D {
    pub min: glam::Vec3A,
    pub max: glam::Vec3A,
}

impl std::fmt::Display for Bound3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.extract_axis(Axis::X);
        let y = self.extract_axis(Axis::Y);
        let z = self.extract_axis(Axis::Z);
        write!(f, "Bound({:?}, {:?}, {:?})", x, y, z)
    }
}

impl From<[Range<f32>; 3]> for Bound3D {
    fn from(value: [Range<f32>; 3]) -> Self {
        Self {
            min: value.map_axis(|_, x| x.start),
            max: value.map_axis(|_, x| x.end),
        }
    }
}

impl Default for Bound3D {
    fn default() -> Self {
        Self::NO_BOUND
    }
}

impl ExtractAxis for Bound3D {
    type Target = Range<f32>;

    fn extract_axis(&self, axis: Axis) -> Self::Target {
        self.min.extract_axis(axis)..self.max.extract_axis(axis)
    }
}

pub trait LimitRange<R> {
    fn limit(&mut self, axis: Axis, range: R);
}

impl LimitRange<RangeFull> for Bound3D {
    fn limit(&mut self, _: Axis, _: RangeFull) {}
}

impl LimitRange<Range<f32>> for Bound3D {
    fn limit(&mut self, axis: Axis, range: Range<f32>) {
        let Range { start, end } = range;
        self.min.apply_axis(axis, move |value| value.max(start));
        self.max.apply_axis(axis, move |value| value.min(end));
    }
}

impl LimitRange<RangeFrom<f32>> for Bound3D {
    fn limit(&mut self, axis: Axis, range: RangeFrom<f32>) {
        self.min
            .apply_axis(axis, move |value| value.max(range.start));
    }
}

impl LimitRange<RangeTo<f32>> for Bound3D {
    fn limit(&mut self, axis: Axis, range: RangeTo<f32>) {
        self.min.apply_axis(axis, move |value| value.max(range.end));
    }
}

impl Bound3D {
    pub const NO_BOUND: Self = Self {
        min: glam::const_vec3a!([f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY]),
        max: glam::const_vec3a!([f32::INFINITY, f32::INFINITY, f32::INFINITY]),
    };

    pub fn from_world(size: &MapSize) -> Self {
        let xr = ((size.width() as usize) * Chunk::WIDTH) as f32;
        let yr = Chunk::HEIGHT as f32;
        let zr = ((size.height() as usize) * Chunk::WIDTH) as f32;
        Self::from([0.0..xr, 0.0..yr, 0.0..zr])
    }

    pub fn merge_axis(&mut self, axis: Axis, rhs: &Self) {
        self.limit(axis, rhs.extract_axis(axis));
    }

    pub fn shrink_by(&self, extent: glam::Vec2) -> Self {
        let radius = extent.x / 2.0;
        let height = extent.y;
        self.map_axis(|axis, value| match axis {
            Axis::X | Axis::Z => (value.start + radius)..(value.end - radius),
            Axis::Y => value.start..(value.end - height),
        })
    }

    pub fn apply<T, R>(&self, target: T) -> R
    where
        T: ExtractAxis<Target = f32>,
        R: From<[f32; 3]>,
    {
        self.map_axis(|axis, Range { start, end }| {
            let val = target.extract_axis(axis);
            if start < end {
                val.clamp(start, end)
            } else {
                val
            }
        })
    }

    pub fn out_of_bound<T>(&self, target: T) -> bool
    where
        T: ExtractAxis<Target = f32>,
    {
        for axis in Axis::iter() {
            let bound = self.extract_axis(axis);
            let val = target.extract_axis(axis);
            if val < bound.start || val > bound.end {
                return true;
            }
        }
        return false;
    }

    pub fn from_block(center: glam::Vec3A, target: glam::Vec3A) -> Self {
        let limits = LimitMode::from_diff(center, target);
        target.map_axis(move |axis, value| limits.get_axis(axis).calc_block_bound(value))
    }
}

impl BitAnd for Bound3D {
    type Output = Self;

    fn bitand(mut self, rhs: Self) -> Self::Output {
        for axis in Axis::iter() {
            self.merge_axis(axis, &rhs)
        }
        self
    }
}

impl BitAndAssign for Bound3D {
    fn bitand_assign(&mut self, rhs: Self) {
        for axis in Axis::iter() {
            self.merge_axis(axis, &rhs)
        }
    }
}
