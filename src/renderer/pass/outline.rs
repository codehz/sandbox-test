use glium::{uniform, Surface};

use crate::{postprocess_shader_program, renderer::buffers::SurfaceProvider};

use super::{pp::PostProcessPass, Pass, PassContext};

pub struct OutlinePass {
    program: glium::Program,
    buffer: PostProcessPass,
}

impl Pass for OutlinePass {
    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self {
            program: postprocess_shader_program!(display, "outline")?,
            buffer: PostProcessPass::new(display)?,
        })
    }

    #[inline(always)]
    fn prepare(&mut self, _context: PassContext, _display: &glium::Display) {}

    fn process(
        &self,
        _context: PassContext,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let (color, normal, position) = provider.get_buffer().get_gbuffer_sampled();
        let mut surface = provider.get_outline_surface(display)?;
        surface.clear_color(0.0, 0.0, 0.0, 1.0);
        let (vertex, index) = (&self.buffer).into();
        let uniforms = uniform! {
            color_sample: color,
            normal_sample: normal,
            position_sample: position,
        };
        let draw_parameters = Default::default();
        surface.draw(vertex, index, &self.program, &uniforms, &draw_parameters)?;
        Ok(())
    }
}
