use std::{borrow::Cow, rc::Rc};

use bevy_ecs::prelude::*;
use glium::{
    backend::{Context, Facade},
    implement_vertex,
    index::PrimitiveType,
    texture::{ClientFormat, MipmapsOption, RawImage2d},
    uniform,
    uniforms::{
        MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerBehavior, SamplerWrapFunction,
    },
    Blend, BlitTarget, DrawParameters, IndexBuffer, Program, Rect, Surface, Texture2d,
    VertexBuffer,
};
use imgui::{BackendFlags, DrawCmdParams, DrawData, ImString, TextureId, Textures};

use crate::{renderer::buffers::SurfaceProvider, shader_program};

use super::{Pass, PassContext};

pub trait UiConcept: Send + Sync {
    fn render(&self, world: &mut World, ui: &mut imgui::Ui);
}

impl<F> UiConcept for F
where
    F: Fn(&mut World, &mut imgui::Ui) + Send + Sync,
{
    fn render(&self, world: &mut World, ui: &mut imgui::Ui) {
        self(world, ui)
    }
}

pub struct Texture {
    pub texture: Rc<Texture2d>,
    pub sampler: SamplerBehavior,
}

pub struct UiPass {
    program: Program,
    font_texture: Texture,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

implement_vertex!(UiVertex, pos, uv, col);

impl UiPass {
    fn lookup_texture<'a>(
        &'a self,
        textures: &'a Textures<Texture>,
        texture_id: TextureId,
    ) -> anyhow::Result<&'a Texture> {
        if texture_id.id() == usize::MAX {
            Ok(&self.font_texture)
        } else if let Some(texture) = textures.get(texture_id) {
            Ok(texture)
        } else {
            Err(anyhow::format_err!("bad texture id: {:?}", texture_id))
        }
    }
    fn render<'a>(
        &'a self,
        textures: &'a Textures<Texture>,
        display: &glium::Display,
        surface: &mut impl Surface,
        draw_data: &DrawData,
    ) -> anyhow::Result<()> {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];
        if !(fb_width > 0.0 && fb_height > 0.0) {
            return Ok(());
        }
        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

        let matrix = [
            [(2.0 / (right - left)), 0.0, 0.0, 0.0],
            [0.0, (2.0 / (top - bottom)), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [
                (right + left) / (left - right),
                (top + bottom) / (bottom - top),
                0.0,
                1.0,
            ],
        ];

        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = VertexBuffer::immutable(display, unsafe {
                draw_list.transmute_vtx_buffer::<UiVertex>()
            })?;
            let idx_buffer = IndexBuffer::immutable(
                display,
                PrimitiveType::TrianglesList,
                draw_list.idx_buffer(),
            )?;
            for cmd in draw_list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                                ..
                            },
                    } => {
                        let clip_rect = [
                            (clip_rect[0] - clip_off[0]) * clip_scale[0],
                            (clip_rect[1] - clip_off[1]) * clip_scale[1],
                            (clip_rect[2] - clip_off[0]) * clip_scale[0],
                            (clip_rect[3] - clip_off[1]) * clip_scale[1],
                        ];

                        if clip_rect[0] < fb_width
                            && clip_rect[1] < fb_height
                            && clip_rect[2] >= 0.0
                            && clip_rect[3] >= 0.0
                        {
                            let texture = self.lookup_texture(textures, texture_id)?;

                            surface.draw(
                                vtx_buffer
                                    .slice(vtx_offset..)
                                    .expect("Invalid vertex buffer range"),
                                idx_buffer
                                    .slice(idx_offset..(idx_offset + count))
                                    .expect("Invalid index buffer range"),
                                &self.program,
                                &uniform! {
                                    matrix: matrix,
                                    tex: Sampler(texture.texture.as_ref(), texture.sampler)
                                },
                                &DrawParameters {
                                    blend: Blend::alpha_blending(),
                                    scissor: Some(Rect {
                                        left: f32::max(0.0, clip_rect[0]).floor() as u32,
                                        bottom: f32::max(0.0, fb_height - clip_rect[3]).floor()
                                            as u32,
                                        width: (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                                        height: (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                                    }),
                                    ..DrawParameters::default()
                                },
                            )?;
                        }
                    }
                    imgui::DrawCmd::ResetRenderState => {}
                    imgui::DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        use imgui::internal::RawWrapper;
                        callback(draw_list.raw(), raw_cmd)
                    },
                }
            }
        }
        Ok(())
    }
}

impl Pass for UiPass {
    fn new(context: &mut PassContext<'_>, display: &glium::Display) -> anyhow::Result<Self> {
        let mut ctx = context
            .world
            .get_non_send_resource_mut::<imgui::Context>()
            .unwrap();
        ctx.set_renderer_name(Some(ImString::new("sandbox-ui-pass")));
        ctx.io_mut()
            .backend_flags
            .insert(BackendFlags::RENDERER_HAS_VTX_OFFSET);
        Ok(Self {
            program: shader_program!(display, "ui")?,
            font_texture: upload_font_texture(ctx.fonts(), display.get_context())?,
        })
    }

    fn prepare(&mut self, context: &mut PassContext<'_>, display: &glium::Display) {
        let mut ctx = context.world.get_non_send_resource_mut::<imgui::Context>().unwrap();
        let window = display.gl_window();
        let window = window.window();
        let scale_factor = window.scale_factor();
        let size = window.inner_size().to_logical::<f32>(scale_factor);
        let mut io = ctx.io_mut();
        io.display_size = [size.width, size.height];
        io.display_framebuffer_scale = [scale_factor as f32, scale_factor as f32];
    }

    fn process(
        &self,
        context: &mut PassContext<'_>,
        provider: &SurfaceProvider,
        display: &glium::Display,
    ) -> anyhow::Result<()> {
        let mut ctx: imgui::Context = context.world.remove_non_send().unwrap();
        let textures: Textures<Texture> = context.world.remove_non_send().unwrap();
        let concept: Box<dyn UiConcept> = context.world.remove_resource().unwrap();
        let res = {
            let mut ui = ctx.frame();
            concept.render(context.world, &mut ui);
            let mut frame = display.draw();
            let fb = provider.get_last_postprocess_surface(display)?;
            let (width, height) = frame.get_dimensions();
            frame.blit_from_simple_framebuffer(
                &fb,
                &Rect {
                    left: 0,
                    bottom: 0,
                    width,
                    height,
                },
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: width as i32,
                    height: height as i32,
                },
                MagnifySamplerFilter::Nearest,
            );
            let draw_data = ui.render();
            self.render(&textures, display, &mut frame, draw_data)
                .and_then(move |_| {
                    frame.finish()?;
                    Ok(())
                })
        };
        context.world.insert_non_send(ctx);
        context.world.insert_non_send(textures);
        context.world.insert_resource(concept);
        res
    }
}

fn upload_font_texture(
    mut fonts: imgui::FontAtlasRefMut,
    ctx: &Rc<Context>,
) -> anyhow::Result<Texture> {
    let texture = fonts.build_rgba32_texture();
    let data = RawImage2d {
        data: Cow::Borrowed(texture.data),
        width: texture.width,
        height: texture.height,
        format: ClientFormat::U8U8U8U8,
    };
    let font_texture = Texture2d::with_mipmaps(ctx, data, MipmapsOption::NoMipmap)?;
    fonts.tex_id = TextureId::from(usize::MAX);
    Ok(Texture {
        texture: Rc::new(font_texture),
        sampler: SamplerBehavior {
            minify_filter: MinifySamplerFilter::Linear,
            magnify_filter: MagnifySamplerFilter::Linear,
            wrap_function: (
                SamplerWrapFunction::BorderClamp,
                SamplerWrapFunction::BorderClamp,
                SamplerWrapFunction::BorderClamp,
            ),
            ..Default::default()
        },
    })
}
