use super::{chunk::Chunk, ChunkPos, SizeTuple};

use rayon::prelude::*;

pub trait Generator {
    fn init(&mut self, size: SizeTuple);
    fn generate(&self, pos: ChunkPos) -> Box<Chunk>;
}

pub mod empty {
    use super::*;

    pub struct EmptyGenerator;

    impl Generator for EmptyGenerator {
        fn init(&mut self, _size: SizeTuple) {}

        fn generate(&self, _pos: ChunkPos) -> Box<Chunk> {
            Chunk::empty()
        }
    }
}

pub mod flat {
    use crate::world::block::Block;

    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct Span(pub Option<&'static Block>, pub u8);

    pub struct FlatGenerator {
        data: Vec<Option<&'static Block>>,
    }

    impl FlatGenerator {
        pub fn new(data: &[Span]) -> Self {
            let mut arr = Vec::new();
            for span in data {
                arr.reserve(span.1 as usize);
                for _ in 0..span.1 {
                    arr.push(span.0);
                }
            }
            assert!(arr.len() < 256, "too many content");
            Self { data: arr }
        }

        fn fetch_block(&self, pos: u8) -> Option<&'static Block> {
            let pos = pos as usize;
            if pos < self.data.len() {
                self.data[pos]
            } else {
                None
            }
        }
    }

    impl Generator for FlatGenerator {
        fn init(&mut self, _size: SizeTuple) {}

        fn generate(&self, _pos: ChunkPos) -> Box<Chunk> {
            let mut ret = Chunk::empty();
            ret.par_iter_mut().for_each(|(pos, blk)| {
                let (_, y, _) = pos.into();
                if let Some(new) = self.fetch_block(y) {
                    blk.replace(new);
                }
            });
            ret
        }
    }
}

pub mod noise {
    use rayon::iter::ParallelIterator;

    use crate::world::{block::Block, chunk::Chunk, ChunkPos};

    use super::Generator;

    pub struct NoiseGenerator<F, F2>
    where
        F: noise::NoiseFn<[f64; 3]> + Send + Sync,
        F2: noise::NoiseFn<[f64; 3]> + Send + Sync,
    {
        noise_fn: F,
        color_noise_fn: F2,
    }

    impl<F, F2> NoiseGenerator<F, F2>
    where
        F: noise::NoiseFn<[f64; 3]> + Send + Sync,
        F2: noise::NoiseFn<[f64; 3]> + Send + Sync,
    {
        pub fn new(noise_fn: F, color_noise_fn: F2) -> Self {
            Self {
                noise_fn,
                color_noise_fn,
            }
        }

        fn get_level(&self, pos: [f64; 3]) -> f32 {
            self.noise_fn.get(pos) as f32 * 64.0 + 32.0
        }

        fn get_color_level(&self, pos: [f64; 3]) -> &'static Block {
            use crate::world::block::constants::*;
            let selection = [
                GREEN_BLOCK,
                RED_BLOCK,
                BLUE_BLOCK,
                PURPLE_BLOCK,
                YELLOW_BLOCK,
                AQUA_BLOCK,
            ];
            let raw = self.color_noise_fn.get(pos);
            let idx = (raw * 6.0 + 3.0).clamp(0.0, 5.9).floor() as usize;
            selection[idx]
        }
    }

    impl<F, F2> Generator for NoiseGenerator<F, F2>
    where
        F: noise::NoiseFn<[f64; 3]> + Send + Sync,
        F2: noise::NoiseFn<[f64; 3]> + Send + Sync,
    {
        fn init(&mut self, _size: crate::world::SizeTuple) {}

        fn generate(&self, chunk_pos: ChunkPos) -> Box<Chunk> {
            let mut ret = Chunk::empty();
            ret.par_iter_mut().for_each(|(block_pos, blk)| {
                let pos = chunk_pos + block_pos;
                // log::info!("{}+{} = {}", chunk_pos, block_pos, pos);
                if pos.y > (Chunk::HEIGHT - 4) as f32 {
                    return;
                }
                let v = pos.y;
                let level = self.get_level([pos.z as f64, pos.y as f64, pos.x as f64]);
                if level > v {
                    blk.replace(self.get_color_level([pos.x as f64, pos.y as f64, pos.z as f64]));
                }
            });
            ret
        }
    }
}
