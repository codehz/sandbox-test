use range_check::{Check, OutOfRangeError};
use rayon::prelude::*;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::{Add, Index, IndexMut},
    sync::atomic::{AtomicBool, Ordering},
    usize,
};

use crate::common::{color::Color, direction::Direction};

use super::block::{Block, BlockType};

const WIDTH: usize = 8;
const HEIGHT: usize = 64;

pub const CHUNK_SIZE: usize = WIDTH * WIDTH * HEIGHT;

/// 16 * 16 * 64 block storage
#[derive(Debug)]
pub struct Chunk {
    dirty: AtomicBool,
    data: [Option<&'static Block>; CHUNK_SIZE],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockSubPos(usize);

impl BlockSubPos {
    fn from_xyz_unchecked(x: usize, y: usize, z: usize) -> Self {
        Self(x + WIDTH * (z + WIDTH * y))
    }

    pub fn new(x: u8, y: u8, z: u8) -> Self {
        (x, y, z).try_into().unwrap()
    }

    fn as_index(self) -> usize {
        self.0
    }
}

impl TryFrom<(u8, u8, u8)> for BlockSubPos {
    type Error = OutOfRangeError<usize>;

    fn try_from((x, y, z): (u8, u8, u8)) -> Result<Self, Self::Error> {
        let (x, y, z) = (x as usize, y as usize, z as usize);
        Ok(Self::from_xyz_unchecked(
            x.check_range(0..WIDTH)?,
            y.check_range(0..HEIGHT)?,
            z.check_range(0..WIDTH)?,
        ))
    }
}

impl Into<(u8, u8, u8)> for BlockSubPos {
    fn into(self) -> (u8, u8, u8) {
        let x = self.0 % WIDTH;
        let z = self.0 / WIDTH % WIDTH;
        let y = self.0 / WIDTH / WIDTH;
        (x as u8, y as u8, z as u8)
    }
}

impl Into<(u8, u8, u8)> for &BlockSubPos {
    fn into(self) -> (u8, u8, u8) {
        (*self).into()
    }
}

impl fmt::Debug for BlockSubPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.into();
        write!(f, "({}, {}, {})", x, y, z)
    }
}

impl fmt::Display for BlockSubPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.into();
        write!(f, "({}, {}, {})", x, y, z)
    }
}

impl Chunk {
    pub const WIDTH: usize = WIDTH;
    pub const HEIGHT: usize = HEIGHT;

    fn mark_dirty(&mut self) {
        *self.dirty.get_mut() = true;
    }

    pub const fn max_face_count() -> usize {
        3 * WIDTH * WIDTH * HEIGHT + WIDTH * WIDTH + 2 * WIDTH * HEIGHT
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = (BlockSubPos, &'_ Option<&'static Block>)> {
        self.data
            .iter()
            .enumerate()
            .map(|(id, blk)| (BlockSubPos(id), blk))
    }

    pub fn par_iter(
        &self,
    ) -> impl '_ + ParallelIterator<Item = (BlockSubPos, &'_ Option<&'static Block>)> {
        self.data
            .par_iter()
            .enumerate()
            .map(|(id, blk)| (BlockSubPos(id), blk))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl '_ + Iterator<Item = (BlockSubPos, &'_ mut Option<&'static Block>)> {
        self.mark_dirty();
        self.data
            .iter_mut()
            .enumerate()
            .map(|(id, blk)| (BlockSubPos(id), blk))
    }

    pub fn par_iter_mut(
        &mut self,
    ) -> impl '_ + ParallelIterator<Item = (BlockSubPos, &'_ mut Option<&'static Block>)> {
        self.mark_dirty();
        self.data
            .par_iter_mut()
            .enumerate()
            .map(|(id, blk)| (BlockSubPos(id), blk))
    }

    pub fn iter_solid(&self) -> impl '_ + Iterator<Item = (BlockSubPos, Color)> {
        self.iter().filter_map(|(id, blk)| {
            blk.map(|blk| match blk.data {
                BlockType::Solid { color } => (id, color),
            })
        })
    }

    pub fn par_iter_solid(&self) -> impl '_ + ParallelIterator<Item = (BlockSubPos, Color)> {
        self.par_iter().filter_map(|(id, blk)| {
            // log::info!("id -> {} = {:?}", id, id);
            blk.map(|blk| match blk.data {
                BlockType::Solid { color } => (id, color),
            })
        })
    }

    pub fn dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    pub fn mark_clean(&self) {
        self.dirty.store(false, Ordering::SeqCst);
    }

    pub fn empty() -> Box<Self> {
        let mut ret = box Chunk {
            dirty: false.into(),
            data: [None; CHUNK_SIZE],
        };
        ret.mark_dirty();
        ret
    }
}

impl Index<BlockSubPos> for Chunk {
    type Output = Option<&'static Block>;

    fn index(&self, index: BlockSubPos) -> &Self::Output {
        &self.data[index.as_index()]
    }
}

impl IndexMut<BlockSubPos> for Chunk {
    fn index_mut(&mut self, index: BlockSubPos) -> &mut Self::Output {
        *self.dirty.get_mut() = true;
        &mut self.data[index.as_index()]
    }
}

impl Add<Direction> for BlockSubPos {
    type Output = Option<BlockSubPos>;

    fn add(self, rhs: Direction) -> Self::Output {
        let (x, y, z) = self.into();
        // log::info!("{}, {}, {}, {}", self.0, x, y, z);
        match rhs {
            Direction::North => {
                if z == 0 {
                    None
                } else {
                    Some(Self::new(x, y, z - 1))
                }
            }
            Direction::South => {
                if z as usize >= WIDTH - 1 {
                    None
                } else {
                    Some(Self::new(x, y, z + 1))
                }
            }
            Direction::East => {
                if x as usize >= WIDTH - 1 {
                    None
                } else {
                    Some(Self::new(x + 1, y, z))
                }
            }
            Direction::West => {
                if x == 0 {
                    None
                } else {
                    Some(Self::new(x - 1, y, z))
                }
            }
            Direction::Up => {
                if y as usize >= HEIGHT - 1 {
                    None
                } else {
                    Some(Self::new(x, y + 1, z))
                }
            }
            Direction::Down => {
                if y == 0 {
                    None
                } else {
                    Some(Self::new(x, y - 1, z))
                }
            }
        }
    }
}
