use std::{fmt, ops::Range};

use strum::IntoEnumIterator;

use super::{
    aabb::AABB,
    axis::{Axis, HasAxisMut, HasAxisMutExt},
};
use super::{
    axis::{ExtractAxis, MapAxisExt},
    bound3d::Bound3D,
};

#[derive(Clone, Copy)]
pub struct VoxelBound {
    position: f32,
    area: f32,
}

fn filter_voxel_edge(val: f32) -> Option<f32> {
    if val.fract() == 0.0 {
        Some(val)
    } else {
        None
    }
}

fn float_cmp(val: f32) -> std::cmp::Ordering {
    if val.abs() < 0.01 {
        return std::cmp::Ordering::Equal;
    }
    match val.partial_cmp(&0.0f32) {
        None => std::cmp::Ordering::Equal,
        Some(x) => x,
    }
}

fn discard_voxel_edge(Range { start, end }: Range<f32>) -> (f32, f32) {
    let start = filter_voxel_edge(start);
    let end = filter_voxel_edge(end);
    match (start, end) {
        (None, None) | (Some(_), Some(_)) => (0.0, f32::NAN),
        (Some(start), None) => (-1.0, start),
        (None, Some(end)) => (1.0, end),
    }
}

impl Into<Range<f32>> for VoxelBound {
    fn into(self) -> Range<f32> {
        use std::cmp::Ordering::*;
        match float_cmp(self.area) {
            Greater => self.position..f32::INFINITY,
            Less => f32::NEG_INFINITY..self.position,
            Equal => f32::NEG_INFINITY..f32::INFINITY,
        }
    }
}

impl fmt::Display for VoxelBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VoxelBound({}@{})", self.position, self.area)
    }
}

impl fmt::Debug for VoxelBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}@{})", self.position, self.area)
    }
}

impl VoxelBound {
    pub fn from_aabb(aabb: AABB) -> [Self; 3] {
        aabb.map_axis(|axis, value| {
            let (direction, position) = discard_voxel_edge(value);
            VoxelBound {
                position,
                area: axis
                    .rest()
                    .iter()
                    .map(move |&axis| aabb.extract_axis(axis))
                    .map(|rng| (rng.end - rng.start).abs())
                    .fold(direction, |a, b| a * b),
            }
            .filter_small()
        })
    }

    pub fn from_aabb_axis(aabb: AABB, axis: Axis) -> Self {
        let (direction, position) = discard_voxel_edge(aabb.extract_axis(axis));
        VoxelBound {
            position,
            area: axis
                .rest()
                .iter()
                .map(move |&axis| aabb.extract_axis(axis))
                .map(|rng| (rng.end - rng.start).abs())
                .fold(direction, |a, b| a * b),
        }
        .filter_small()
    }

    fn filter_small(self) -> Self {
        if float_cmp(self.area) == std::cmp::Ordering::Equal {
            Self::default()
        } else {
            self
        }
    }

    pub fn area(self) -> f32 {
        self.area
    }

    pub fn merge(&mut self, rhs: Self) {
        use std::cmp::Ordering::*;
        if rhs.position.is_nan() {
            return;
        }
        self.area += rhs.area;
        match float_cmp(self.area) {
            Greater => self.position = self.position.max(rhs.position),
            Less => self.position = self.position.min(rhs.position),
            Equal => self.position = f32::NAN,
        }
    }
}

impl Default for VoxelBound {
    fn default() -> Self {
        Self {
            position: f32::NAN,
            area: 0.0,
        }
    }
}

pub trait VoxelBoundExt:
    HasAxisMut<Target = VoxelBound> + ExtractAxis<Target = VoxelBound>
{
    fn merge(&mut self, rhs: Self);

    fn find_max_area(self) -> Option<(Axis, Range<f32>)>;

    fn to_bound3d(self) -> Bound3D;
}

impl<X: HasAxisMut<Target = VoxelBound> + ExtractAxis<Target = VoxelBound>> VoxelBoundExt for X {
    fn merge(&mut self, rhs: Self) {
        for axis in Axis::iter() {
            self.for_axis(axis, |value| {
                use std::cmp::Ordering::*;
                let rhs = rhs.get_axis(axis);
                if rhs.position.is_nan() {
                    return;
                }
                value.area += rhs.area;
                match float_cmp(value.area) {
                    Greater => value.position = value.position.max(rhs.position),
                    Less => value.position = value.position.min(rhs.position),
                    Equal => value.position = f32::NAN,
                }
            });
        }
    }

    fn find_max_area(self) -> Option<(Axis, Range<f32>)> {
        let mut max_area = 0.0f32;
        let mut ret = None;
        for axis in Axis::iter() {
            let value = self.extract_axis(axis);
            let abs_area = value.area.abs();
            if value.area.is_finite() && max_area < abs_area {
                max_area = abs_area;
                ret = Some((axis, value.into()));
            }
        }
        ret
    }

    fn to_bound3d(self) -> Bound3D {
        self.map_axis(|_, value| Into::<Range<f32>>::into(value))
    }
}
