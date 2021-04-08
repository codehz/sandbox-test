use glium::{implement_uniform_block, uniform, Surface};

use crate::{postprocess_shader_program, renderer::buffers::SurfaceProvider};

use super::{pp::PostProcessPass, Pass, PassContext};

pub struct StrengthenPass {
    program: glium::Program,
    buffer: PostProcessPass,
}

#[derive(Debug, Clone, Copy)]
struct CameraBlock {
    near: f32,
    far: f32,
}

implement_uniform_block!(CameraBlock, near, far);

impl Pass for StrengthenPass {
    fn new(_context: &mut PassContext<'_>, display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self {
            program: postprocess_shader_program!(display, "strengthen")?,
            buffer: PostProcessPass::new(display)?,
        })
    }

    #[inline(always)]
    fn prepare(&mut self, _context: &mut PassContext, _display: &glium::Display) {}

    fn process(
        &self,
        context: &mut PassContext,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let strengthen = {
            let range = context.camera().soft_range.clone();
            CameraBlock {
                near: range.start,
                far: range.end,
            }
        };
        let sprite = provider.get_buffer().get_sprite_sampled();
        let camera_block = glium::uniforms::UniformBuffer::new(display, strengthen)?;
        let color = provider.get_last_postprocess_sample();
        let altdepth = provider.get_aux_sample();
        let mut surface = provider.get_postprocess_surface(display)?;
        surface.clear_color(0.0, 0.0, 0.0, 1.0);
        let (vertex, index) = (&self.buffer).into();
        let uniforms = uniform! {
            color_sample: color,
            aux_sample: altdepth,
            sprite_sample: sprite,
            block: &camera_block,
        };
        let draw_parameters = Default::default();
        surface.draw(vertex, index, &self.program, &uniforms, &draw_parameters)?;
        Ok(())
    }
}
