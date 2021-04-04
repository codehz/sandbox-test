use std::{
    convert::TryInto,
    fmt,
    ops::{Add, Index, IndexMut},
};

use enum_map::Enum;
use range_check::Check;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::math::aabb::AABB;

use self::{
    block::Block,
    chunk::{BlockSubPos, Chunk},
    generator::Generator,
};

pub mod block;
pub mod block_iter;
pub mod chunk;
pub mod generator;

type SizeType = u8;
type SizeTuple = (SizeType, SizeType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkPos(pub SizeType, pub SizeType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum, EnumIter)]
pub enum ChunkNeighbor {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct MapSize(SizeTuple);

impl ChunkNeighbor {
    fn as_offsets(self) -> (i32, i32) {
        match self {
            ChunkNeighbor::North => (0, -1),
            ChunkNeighbor::South => (0, 1),
            ChunkNeighbor::East => (1, 0),
            ChunkNeighbor::West => (-1, 0),
        }
    }
}

impl MapSize {
    #[cfg(test)]
    pub fn new(size: SizeTuple) -> Self {
        Self(size)
    }

    pub fn width(&self) -> SizeType {
        self.0 .0
    }
    pub fn height(&self) -> SizeType {
        self.0 .1
    }

    pub fn to_world_size(&self) -> glam::UVec3 {
        glam::uvec3(
            (self.width() as u32) * (Chunk::WIDTH as u32),
            Chunk::HEIGHT as u32,
            (self.height() as u32) * (Chunk::WIDTH as u32),
        )
    }

    fn get_index(&self, ChunkPos(x, z): ChunkPos) -> usize {
        let (w, h) = self.0;
        x.check_range(..w).expect("x out of range");
        z.check_range(..h).expect("z out of range");
        let w = w as usize;
        let (x, z) = (x as usize, z as usize);
        z * w + x
    }

    fn from_index(&self, index: usize) -> ChunkPos {
        let (w, h) = self.0;
        let max = (w as usize) * (h as usize);
        index.check_range(0..max).expect("index out of range");
        let w = w as usize;
        ChunkPos((index % w) as u8, (index / w) as u8)
    }

    fn pos_neighbor(self, pos: ChunkPos) -> impl Iterator<Item = ChunkPos> {
        ChunkNeighbor::iter().filter_map(move |dir| pos.get_neighbor(self, dir))
    }

    pub fn convert_pos(self, pos: glam::UVec3) -> Option<(ChunkPos, BlockSubPos)> {
        if pos.y.check_range(0..(Chunk::HEIGHT as u32)).is_err() {
            return None;
        }
        let cx = pos.x / (Chunk::WIDTH as u32);
        let cz = pos.z / (Chunk::WIDTH as u32);
        if cx.check_range(0..(self.width() as u32)).is_err()
            || cz.check_range(0..(self.height() as u32)).is_err()
        {
            return None;
        }
        let (cx, cz) = (cx as u8, cz as u8);
        let (bx, by, bz) = (
            pos.x % (Chunk::WIDTH as u32),
            pos.y,
            pos.z % (Chunk::WIDTH as u32),
        );
        match (bx as u8, by as u8, bz as u8).try_into() {
            Ok(block_pos) => Some((ChunkPos(cx, cz), block_pos)),
            Err(_) => None,
        }
    }
}

impl Into<(u8, u8)> for MapSize {
    fn into(self) -> (u8, u8) {
        self.0
    }
}

pub struct Map {
    chunks: Vec<Box<Chunk>>,
    size: MapSize,
}

impl ChunkPos {
    pub fn get_neighbor(self, size: MapSize, direction: ChunkNeighbor) -> Option<Self> {
        let (a, b) = direction.as_offsets();
        match (a, b, self.0, self.1) {
            (-1, _, 0, _) => None,
            (1, _, x, _) if x == size.width() - 1 => None,
            (_, -1, _, 0) => None,
            (_, 1, _, y) if y == size.height() - 1 => None,
            (a, b, x, y) => Some(ChunkPos((x as i32 + a) as u8, (y as i32 + b) as u8)),
        }
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(x, z) = self;
        write!(f, "({}, {})", x, z)
    }
}

impl Map {
    pub fn new<G: Generator>(size: SizeTuple, mut generator: G) -> Self {
        size.0.check_range(1..).expect("size should large than 0");
        size.1.check_range(1..).expect("size should large than 0");
        let mut chunks = Vec::with_capacity((size.0 as usize) * (size.1 as usize));
        generator.init(size);
        for j in 0..size.1 {
            for i in 0..size.0 {
                chunks.push(generator.generate(ChunkPos(i, j)));
            }
        }
        let size = MapSize(size);
        Self { chunks, size }
    }

    pub fn size(&self) -> MapSize {
        self.size
    }

    pub fn in_range(&self, pos: ChunkPos) -> bool {
        let (w, h) = self.size.into();
        pos.0.check_range(..w).is_ok() && pos.1.check_range(..h).is_ok()
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item = (ChunkPos, &Chunk)> {
        let size = self.size;
        self.chunks
            .par_iter()
            .enumerate()
            .map(move |(i, c)| (size.from_index(i), c.as_ref()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (ChunkPos, &Chunk)> {
        let size = self.size;
        self.chunks
            .iter()
            .enumerate()
            .map(move |(i, c)| (size.from_index(i), c.as_ref()))
    }

    pub fn iter_neighbor(&self, pos: ChunkPos) -> impl Iterator<Item = (ChunkPos, &Chunk)> {
        self.size
            .pos_neighbor(pos)
            .map(move |pos| (pos, &self[pos]))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ChunkPos, &mut Chunk)> {
        let size = self.size;
        self.chunks
            .iter_mut()
            .enumerate()
            .map(move |(i, c)| (size.from_index(i), c.as_mut()))
    }

    pub fn scan_aabb(&self, aabb: AABB) -> impl Iterator<Item = (glam::Vec3A, &Block)> {
        let size = self.size;
        aabb.iter_pos()
            .filter_map(move |pos| size.convert_pos(pos))
            .filter_map(move |(chunk_pos, block_pos)| {
                self[chunk_pos][block_pos].map(move |x| (chunk_pos + block_pos, x))
            })
    }
}

impl Add<BlockSubPos> for ChunkPos {
    type Output = glam::Vec3A;

    fn add(self, rhs: BlockSubPos) -> Self::Output {
        let (x, y, z) = rhs.into();
        let ChunkPos(mx, mz) = self;
        let (x, y, z, mx, mz) = (x as f32, y as f32, z as f32, mx as f32, mz as f32);
        let w = Chunk::WIDTH as f32;
        glam::vec3a(mx * w + x, y, mz * w + z)
    }
}

impl Index<ChunkPos> for Map {
    type Output = Chunk;

    fn index(&self, index: ChunkPos) -> &Self::Output {
        &self.chunks[self.size.get_index(index)]
    }
}

impl IndexMut<ChunkPos> for Map {
    fn index_mut(&mut self, index: ChunkPos) -> &mut Self::Output {
        let idx = self.size.get_index(index);
        &mut self.chunks[idx]
    }
}
