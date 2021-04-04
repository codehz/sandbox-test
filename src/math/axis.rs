use std::cmp::Ordering;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::components::{Position, Velocity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn rest(self) -> [Self; 2] {
        use Axis::*;
        match self {
            X => [Y, Z],
            Y => [X, Z],
            Z => [X, Y],
        }
    }

    pub fn generate_array<T, F>(mut f: F) -> [T; 3]
    where
        T: Sized,
        F: FnMut(Axis) -> T,
    {
        let mut arr = std::mem::MaybeUninit::<T>::uninit_array();
        for axis in Axis::iter() {
            let ptr = arr.get_axis_mut(axis).as_mut_ptr();
            let ret = f(axis);
            unsafe {
                ptr.write(ret);
            }
        }
        let arr: [T; 3] = unsafe { arr.as_ptr().cast::<[T; 3]>().read() };
        arr
    }

    pub fn generate<R, T, F>(f: F) -> R
    where
        T: Sized,
        F: FnMut(Axis) -> T,
        R: From<[T; 3]>,
    {
        Self::generate_array(f).into()
    }
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

pub trait ExtractAxis {
    type Target;

    fn extract_axis(&self, axis: Axis) -> Self::Target;
}

pub trait HasAxis {
    type Target;

    fn get_axis(&self, axis: Axis) -> &Self::Target;
}

pub trait HasAxisMut: HasAxis {
    fn get_axis_mut(&mut self, axis: Axis) -> &mut Self::Target;
    #[inline(always)]
    fn set_axis(&mut self, axis: Axis, value: Self::Target) {
        *self.get_axis_mut(axis) = value;
    }
}

pub trait DiffAxisExt: ExtractAxis
where
    Self::Target: PartialEq,
{
    fn eq_axis<R>(&self, rhs: &R) -> [bool; 3]
    where
        R: ExtractAxis<Target = Self::Target>;
    fn partial_cmp_axis<R>(&self, rhs: &R) -> [Option<Ordering>; 3]
    where
        R: ExtractAxis<Target = Self::Target>,
        Self::Target: PartialOrd;
    fn cmp_axis<R>(&self, rhs: &R) -> [Ordering; 3]
    where
        R: ExtractAxis<Target = Self::Target>,
        Self::Target: Ord;
}

pub trait SortAxisExt: ExtractAxis {
    type Output: Iterator<Item = Axis>;
    fn sort_axis<F>(&self, f: F) -> Self::Output
    where
        F: FnMut(<Self as ExtractAxis>::Target, <Self as ExtractAxis>::Target) -> bool;
}

pub trait MapAxisExt: ExtractAxis {
    fn map_axis<R, T, F>(&self, f: F) -> R
    where
        R: From<[T; 3]>,
        F: FnMut(Axis, Self::Target) -> T;

    fn adjust_axis<R, F>(&self, axis: Axis, f: F) -> R
    where
        R: From<[Self::Target; 3]>,
        F: FnMut(Self::Target) -> Self::Target;
}

pub trait HasAxisMutExt: HasAxisMut {
    fn apply_axis<F>(&mut self, axis: Axis, f: F)
    where
        F: FnOnce(<Self::Target as ToOwned>::Owned) -> Self::Target,
        Self::Target: ToOwned;
    fn for_axis<F>(&mut self, axis: Axis, f: F)
    where
        F: FnOnce(&mut Self::Target);
}

impl<T> ExtractAxis for T
where
    T: HasAxisMut,
    T::Target: ToOwned,
{
    type Target = <T::Target as ToOwned>::Owned;

    fn extract_axis(&self, axis: Axis) -> Self::Target {
        self.get_axis(axis).to_owned()
    }
}

impl<X> DiffAxisExt for X
where
    <Self as ExtractAxis>::Target: PartialEq,
    Self: ExtractAxis,
{
    fn eq_axis<R>(&self, rhs: &R) -> [bool; 3]
    where
        R: ExtractAxis<Target = Self::Target>,
    {
        let mut arr = [false; 3];
        for axis in Axis::iter() {
            *arr.get_axis_mut(axis) = self.extract_axis(axis).eq(&rhs.extract_axis(axis));
        }
        arr
    }

    fn partial_cmp_axis<R>(&self, rhs: &R) -> [Option<Ordering>; 3]
    where
        R: ExtractAxis<Target = Self::Target>,
        Self::Target: PartialOrd,
    {
        let mut arr = [None; 3];
        for axis in Axis::iter() {
            *arr.get_axis_mut(axis) = self.extract_axis(axis).partial_cmp(&rhs.extract_axis(axis));
        }
        arr
    }

    fn cmp_axis<R>(&self, rhs: &R) -> [Ordering; 3]
    where
        R: ExtractAxis<Target = Self::Target>,
        Self::Target: Ord,
    {
        let mut arr = [Ordering::Equal; 3];
        for axis in Axis::iter() {
            *arr.get_axis_mut(axis) = self.extract_axis(axis).cmp(&rhs.extract_axis(axis));
        }
        arr
    }
}

macro_rules! cmp_and_swap {
    ($f:expr, $arr:expr, $x:expr, $y:expr, $a:expr, $b:expr) => {
        if !$f($x, $y) {
            swap(&mut $x, &mut $y);
            $arr.swap($a, $b);
        }
    };
}

impl<X> SortAxisExt for X
where
    Self: ExtractAxis,
    Self::Target: Copy,
{
    type Output = impl Iterator<Item = Axis> + std::fmt::Debug;

    fn sort_axis<F>(
        &self,
        mut f: F,
    ) -> Self::Output where F: FnMut(<Self as ExtractAxis>::Target, <Self as ExtractAxis>::Target) -> bool {
        use std::mem::swap;
        use Axis::*;
        let mut arr = [X, Y, Z];
        let mut x = self.extract_axis(X);
        let mut y = self.extract_axis(Y);
        let mut z = self.extract_axis(Z);
        cmp_and_swap!(f, arr, x, y, 0, 1);
        cmp_and_swap!(f, arr, y, z, 1, 2);
        cmp_and_swap!(f, arr, x, y, 0, 1);
        std::array::IntoIter::new(arr)
    }
}

impl<X> MapAxisExt for X
where
    X: ExtractAxis,
{
    fn map_axis<R, T, F>(&self, mut f: F) -> R
    where
        R: From<[T; 3]>,
        F: FnMut(Axis, Self::Target) -> T,
    {
        Axis::generate(move |axis| f(axis, self.extract_axis(axis)))
    }

    fn adjust_axis<R, F>(&self, axis: Axis, mut f: F) -> R
    where
        R: From<[Self::Target; 3]>,
        F: FnMut(Self::Target) -> Self::Target,
    {
        self.map_axis(move |current, value| if current == axis { f(value) } else { value })
    }
}

impl<T> HasAxisMutExt for T
where
    T: HasAxisMut,
{
    #[inline(always)]
    fn apply_axis<F>(&mut self, axis: Axis, f: F)
    where
        F: FnOnce(<Self::Target as ToOwned>::Owned) -> Self::Target,
        Self::Target: ToOwned,
    {
        let data = self.get_axis_mut(axis);
        *data = f((*data).to_owned());
    }

    #[inline(always)]
    fn for_axis<F>(&mut self, axis: Axis, f: F)
    where
        F: FnOnce(&mut Self::Target),
    {
        let data = self.get_axis_mut(axis);
        f(data);
    }
}

macro_rules! std_has_axis {
    ($type:ty, $target:ty) => {
        impl HasAxis for $type {
            type Target = $target;

            #[inline(always)]
            fn get_axis(&self, axis: Axis) -> &Self::Target {
                match axis {
                    Axis::X => &self.x,
                    Axis::Y => &self.y,
                    Axis::Z => &self.z,
                }
            }
        }

        impl HasAxisMut for $type {
            #[inline(always)]
            fn get_axis_mut(&mut self, axis: Axis) -> &mut Self::Target {
                match axis {
                    Axis::X => &mut self.x,
                    Axis::Y => &mut self.y,
                    Axis::Z => &mut self.z,
                }
            }
        }
    };
}

macro_rules! delegate_has_axis {
    ($type:ty, $target:ty) => {
        impl HasAxis for $type {
            type Target = $target;

            #[inline(always)]
            fn get_axis(&self, axis: Axis) -> &Self::Target {
                self.0.get_axis(axis)
            }
        }

        impl HasAxisMut for $type {
            #[inline(always)]
            fn get_axis_mut(&mut self, axis: Axis) -> &mut Self::Target {
                self.0.get_axis_mut(axis)
            }
            #[inline(always)]
            fn set_axis(&mut self, axis: Axis, value: Self::Target) {
                self.0.set_axis(axis, value)
            }
        }
    };
}

std_has_axis!(glam::Vec3, f32);
std_has_axis!(glam::Vec3A, f32);
std_has_axis!(glam::UVec3, u32);
std_has_axis!(glam::IVec3, i32);
delegate_has_axis!(Position, f32);
delegate_has_axis!(Velocity, f32);

impl HasAxis for glam::BVec3 {
    type Target = bool;

    #[inline(always)]
    fn get_axis(&self, axis: Axis) -> &Self::Target {
        let arr = self.as_ref();
        match axis {
            Axis::X => &arr[0],
            Axis::Y => &arr[1],
            Axis::Z => &arr[2],
        }
    }
}

impl ExtractAxis for glam::BVec3A {
    type Target = bool;

    fn extract_axis(&self, axis: Axis) -> Self::Target {
        let arr = self.as_ref();
        match axis {
            Axis::X => arr[0] != 0,
            Axis::Y => arr[1] != 0,
            Axis::Z => arr[2] != 0,
        }
    }
}

impl<T> HasAxis for [T; 3] {
    type Target = T;

    fn get_axis(&self, axis: Axis) -> &Self::Target {
        match axis {
            Axis::X => &self[0],
            Axis::Y => &self[1],
            Axis::Z => &self[2],
        }
    }
}

impl<T> HasAxisMut for [T; 3] {
    fn get_axis_mut(&mut self, axis: Axis) -> &mut Self::Target {
        match axis {
            Axis::X => &mut self[0],
            Axis::Y => &mut self[1],
            Axis::Z => &mut self[2],
        }
    }
}

impl<T> HasAxis for (T, T, T) {
    type Target = T;

    fn get_axis(&self, axis: Axis) -> &Self::Target {
        match axis {
            Axis::X => &self.0,
            Axis::Y => &self.1,
            Axis::Z => &self.2,
        }
    }
}

impl<T> HasAxisMut for (T, T, T) {
    fn get_axis_mut(&mut self, axis: Axis) -> &mut Self::Target {
        match axis {
            Axis::X => &mut self.0,
            Axis::Y => &mut self.1,
            Axis::Z => &mut self.2,
        }
    }
}
