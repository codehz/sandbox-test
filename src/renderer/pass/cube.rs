use itertools::Itertools;
use rayon::prelude::*;
use std::{collections::BTreeMap, time::Instant};

use super::{Pass, PassContext};
use crate::{
    common::{
        color::Color,
        direction::Direction,
        vertex_cache::{VertexCache, VertexWriter},
    },
    math::axis::MapAxisExt,
    renderer::buffers::SurfaceProvider,
    resources::PickedBlock,
    shader_program,
    world::{
        block::{Block, BlockType},
        chunk::{BlockSubPos, Chunk},
        ChunkPos, Map,
    },
};
use glium::{implement_uniform_block, implement_vertex, uniform, Surface};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy)]
struct FaceInfo {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
    face: u32,
}

implement_vertex!(FaceInfo, position, color, face);

#[derive(Debug, Clone, Copy)]
struct PickedUniformBlock {
    picked_position: [f32; 3],
}

implement_uniform_block!(PickedUniformBlock, picked_position);

impl From<PickedBlock> for PickedUniformBlock {
    fn from(blk: PickedBlock) -> Self {
        Self {
            picked_position: blk.position.map_axis(|_, val| val as f32),
        }
    }
}

pub struct CubePass {
    program: glium::Program,
    chunk_cache: BTreeMap<ChunkPos, VertexCache<FaceInfo>>,
}

fn test_block(chunk: &Chunk, block_pos: BlockSubPos) -> bool {
    if let Some(Block {
        data: BlockType::Solid { .. },
        ..
    }) = chunk[block_pos]
    {
        true
    } else {
        false
    }
}

fn gen_face(
    map: &Map,
    chunk_pos: ChunkPos,
    chunk: &Chunk,
    color: Color,
    block_pos: BlockSubPos,
    direction: Direction,
) -> Option<FaceInfo> {
    match block_pos + direction {
        Some(neighbor) => {
            if test_block(chunk, neighbor) {
                return None;
            }
        }
        None => {
            use crate::world::ChunkNeighbor;
            let (x, y, z) = block_pos.into();
            match direction {
                Direction::West => {
                    if let Some(neighbor_chunk_pos) =
                        chunk_pos.get_neighbor(map.size(), ChunkNeighbor::West)
                    {
                        let neighbor_chunk = &map[neighbor_chunk_pos];
                        if test_block(
                            neighbor_chunk,
                            BlockSubPos::new((Chunk::WIDTH - 1) as u8, y, z),
                        ) {
                            return None;
                        }
                    }
                }
                Direction::East => {
                    if let Some(neighbor_chunk_pos) =
                        chunk_pos.get_neighbor(map.size(), ChunkNeighbor::East)
                    {
                        let neighbor_chunk = &map[neighbor_chunk_pos];
                        if test_block(neighbor_chunk, BlockSubPos::new(0, y, z)) {
                            return None;
                        }
                    }
                }
                Direction::North => {
                    if let Some(neighbor_chunk_pos) =
                        chunk_pos.get_neighbor(map.size(), ChunkNeighbor::North)
                    {
                        let neighbor_chunk = &map[neighbor_chunk_pos];
                        if test_block(
                            neighbor_chunk,
                            BlockSubPos::new(x, y, (Chunk::WIDTH - 1) as u8),
                        ) {
                            return None;
                        }
                    }
                }
                Direction::South => {
                    if let Some(neighbor_chunk_pos) =
                        chunk_pos.get_neighbor(map.size(), ChunkNeighbor::South)
                    {
                        let neighbor_chunk = &map[neighbor_chunk_pos];
                        if test_block(neighbor_chunk, BlockSubPos::new(x, y, 0)) {
                            return None;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Some(FaceInfo {
        position: (chunk_pos + block_pos).into(),
        color: color.into(),
        face: direction as u32,
    })
}

impl Pass for CubePass {
    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self {
            program: shader_program!(display, "cube" with geometry)?,
            chunk_cache: Default::default(),
        })
    }

    fn prepare(&mut self, context: &mut PassContext, display: &glium::Display) {
        let map = context.map();
        map.iter()
            .filter_map(|(chunk_pos, chunk)| {
                let cached = self.chunk_cache.entry(chunk_pos);
                let mut dirty = chunk.dirty()
                    || map
                        .iter_neighbor(chunk_pos)
                        .any(|(_, neighbor)| neighbor.dirty());
                let cache = cached.or_insert_with(|| {
                    dirty = true;
                    VertexCache::new(display, Chunk::max_face_count())
                });
                if dirty {
                    let now = Instant::now();
                    let data: Vec<_> = chunk
                        .par_iter_solid()
                        .flat_map_iter(|(block_pos, color)| {
                            Direction::iter().map(move |direction| (block_pos, color, direction))
                        })
                        .filter_map(|(block_pos, color, direction)| {
                            gen_face(&*map, chunk_pos, chunk, color, block_pos, direction)
                        })
                        .collect();
                    let mut writer = VertexWriter::new(cache);
                    for face in data {
                        writer.write(face);
                    }
                    log::info!("render chunk@{:?} took {:?}", chunk_pos, now.elapsed());
                    Some(chunk)
                } else {
                    None
                }
            })
            .collect_vec()
            .iter()
            .for_each(|x| (**x).mark_clean());
    }

    fn process(
        &self,
        context: &mut PassContext,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let mut frame = provider.get_gbuffer_surface(display)?;
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let picked_block = if let Some(picked) = *context.get_res::<Option<PickedBlock>>() {
            picked.into()
        } else {
            PickedUniformBlock {
                picked_position: [f32::INFINITY, f32::INFINITY, f32::INFINITY],
            }
        };
        let picked_block = glium::uniforms::UniformBuffer::new(display, picked_block)?;

        let uniforms = uniform! {
            view_model: context.view_model,
            perspective: context.perspective,
            picked: &picked_block,
        };

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
        for (_, cache) in &self.chunk_cache {
            frame.draw(
                cache.as_slice(),
                &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                &self.program,
                &uniforms,
                &draw_parameters,
            )?;
        }
        Ok(())
    }
}
