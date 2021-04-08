use glium::{uniform, Surface};

use crate::{postprocess_shader_program, renderer::buffers::SurfaceProvider};

use super::{pp::PostProcessPass, Pass, PassContext};

pub struct DebugPass {
    program: glium::Program,
    buffer: PostProcessPass,
}

impl Pass for DebugPass {
    fn new(_context: &mut PassContext<'_>, display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self {
            program: postprocess_shader_program!(display, "debug_gbuffer")?,
            buffer: PostProcessPass::new(display)?,
        })
    }

    #[inline(always)]
    fn prepare(&mut self, _context: &mut PassContext, _display: &glium::Display) {}

    fn process(
        &self,
        _context: &mut PassContext,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let group = provider.get_buffer();
        let (color, normal, position) = group.get_gbuffer_sampled();
        let sprite = group.get_sprite_sampled();
        let mut surface = display.draw();
        surface.clear_color(0.0, 0.0, 0.0, 1.0);
        let (vertex, index) = (&self.buffer).into();
        let uniforms = uniform! {
            color_sample: color,
            normal_sample: normal,
            position_sample: position,
            sprite_sample: sprite,
        };
        let draw_parameters = Default::default();
        surface.draw(vertex, index, &self.program, &uniforms, &draw_parameters)?;
        surface.finish()?;
        Ok(())
    }
}
