use glium::{implement_vertex, uniform, Surface};

use super::{Pass, PassContext};

use crate::{
    common::vertex_cache::{VertexCache, VertexWriter},
    components::{Position, Sprite},
    renderer::buffers::SurfaceProvider,
    shader_program,
};

#[derive(Debug, Clone, Copy)]
struct SpriteInfo {
    position: (f32, f32, f32),
    color: (f32, f32, f32),
    radius: f32,
}

impl SpriteInfo {
    fn from_entity(position: &Position, sprite: &Sprite) -> Self {
        Self {
            position: position.into(),
            color: sprite.color.into(),
            radius: sprite.radius,
        }
    }
}

implement_vertex!(SpriteInfo, position, color, radius);

pub struct SpritePass {
    program: glium::Program,
    points: Option<VertexCache<SpriteInfo>>,
}

impl Pass for SpritePass {
    fn new(display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self {
            program: shader_program!(display, "sprite" with geometry)?,
            points: None,
        })
    }

    fn prepare(&mut self, context: PassContext<'_>, display: &glium::Display) {
        let points = self
            .points
            .get_or_insert_with(|| VertexCache::new(display, 1024));
        let mut writer = VertexWriter::new(points);
        for (sprite, position) in context.world.query::<(&Sprite, &Position)>() {
            writer.write(SpriteInfo::from_entity(position, sprite));
        }
    }

    fn process(
        &self,
        context: PassContext<'_>,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let mut frame = provider.get_sprite_surface(display)?;

        frame.clear_color(0.0, 0.0, 0.0, 0.0);

        let uniforms = uniform! {
            view_model: context.view_model,
            perspective: context.perspective,
            aspect_ratio: context.aspect_ratio,
        };

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        frame.draw(
            self.points.as_ref().unwrap().as_slice(),
            &glium::index::NoIndices(glium::index::PrimitiveType::Points),
            &self.program,
            &uniforms,
            &draw_parameters,
        )?;

        Ok(())
    }
}
