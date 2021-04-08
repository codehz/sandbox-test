use std::cell::RefCell;

use glium::uniforms::Sampler;

pub struct TextureGroup {
    pub sprite: glium::texture::Texture2d,
    pub color: glium::texture::Texture2d,
    pub normal: glium::texture::Texture2d,
    pub position: glium::texture::Texture2d,
    pub depth: glium::framebuffer::DepthRenderBuffer,
    pub auxtexture: glium::texture::Texture2d,
    pub postprocess: [glium::texture::Texture2d; 2],
    pub flip: RefCell<bool>,
}

impl TextureGroup {
    fn new(disp: &glium::Display, (width, height): (u32, u32)) -> anyhow::Result<Self> {
        Ok(Self {
            sprite: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F32F32F32F32,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            color: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::U8U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            normal: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F16F16F16,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            position: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F16F16F16,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            depth: glium::framebuffer::DepthRenderBuffer::new(
                disp,
                glium::texture::DepthFormat::F32,
                width,
                height,
            )?,
            auxtexture: glium::texture::Texture2d::empty_with_format(
                disp,
                glium::texture::UncompressedFloatFormat::F16F16,
                glium::texture::MipmapsOption::NoMipmap,
                width,
                height,
            )?,
            postprocess: [
                glium::texture::Texture2d::empty_with_format(
                    disp,
                    glium::texture::UncompressedFloatFormat::F16F16F16,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height,
                )?,
                glium::texture::Texture2d::empty_with_format(
                    disp,
                    glium::texture::UncompressedFloatFormat::F16F16F16,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height,
                )?,
            ],
            flip: RefCell::new(false),
        })
    }

    fn get_sprite_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::SimpleFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(display, &self.sprite, &self.depth)
    }

    fn get_gbuffer_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::MultiOutputFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        let color_attachments = [
            ("color", &self.color),
            ("normal", &self.normal),
            ("position", &self.position),
        ];
        glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            display,
            color_attachments.iter().cloned(),
            &self.depth,
        )
    }

    fn get_outline_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::MultiOutputFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        let color_attachments = [
            ("color", self.get_current_postprocess_texture()),
            ("aux", &self.auxtexture),
        ];
        glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(
            display,
            color_attachments.iter().cloned(),
            &self.depth,
        )
    }

    pub fn get_sprite_sampled(&self) -> Sampler<'_, glium::Texture2d> {
        use glium::uniforms::*;
        self.sprite
            .sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .wrap_function(SamplerWrapFunction::Clamp)
    }

    pub fn get_gbuffer_sampled(
        &self,
    ) -> (
        Sampler<'_, glium::Texture2d>,
        Sampler<'_, glium::Texture2d>,
        Sampler<'_, glium::Texture2d>,
    ) {
        use glium::uniforms::*;
        (
            self.color
                .sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .wrap_function(SamplerWrapFunction::Mirror),
            self.normal
                .sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .wrap_function(SamplerWrapFunction::Mirror),
            self.position
                .sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest)
                .wrap_function(SamplerWrapFunction::Mirror),
        )
    }

    fn get_current_postprocess_texture(&self) -> &glium::texture::Texture2d {
        &self.postprocess[*self.flip.borrow() as usize]
    }

    pub fn get_aux_sampled(&self) -> Sampler<'_, glium::Texture2d> {
        use glium::uniforms::*;
        self.auxtexture
            .sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .wrap_function(SamplerWrapFunction::Mirror)
    }

    pub fn get_postprocess_sampled(&self) -> Sampler<'_, glium::Texture2d> {
        use glium::uniforms::*;
        self.get_current_postprocess_texture()
            .sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .wrap_function(SamplerWrapFunction::Mirror)
    }

    fn next_postprocess_texture(&self) {
        let mut flip = self.flip.borrow_mut();
        *flip = !*flip;
    }

    fn get_postprocess_surface<'a>(
        &'a self,
        display: &glium::Display,
    ) -> Result<glium::framebuffer::SimpleFrameBuffer<'a>, glium::framebuffer::ValidationError>
    {
        glium::framebuffer::SimpleFrameBuffer::new(display, self.get_current_postprocess_texture())
    }
}

pub struct SurfaceProvider {
    dimensions: (u32, u32),
    buffer: TextureGroup,
}

impl SurfaceProvider {
    pub fn new(display: &glium::Display) -> anyhow::Result<Self> {
        let dimensions = display.get_framebuffer_dimensions();
        Ok(Self {
            dimensions,
            buffer: TextureGroup::new(display, dimensions)?,
        })
    }

    pub fn get_buffer(&self) -> &TextureGroup {
        &self.buffer
    }

    pub fn verify(&mut self, display: &glium::Display) -> anyhow::Result<()> {
        let dimensions = display.get_framebuffer_dimensions();
        if self.dimensions != dimensions {
            self.buffer = TextureGroup::new(display, dimensions)?;
            self.dimensions = dimensions;
        }
        Ok(())
    }

    pub fn get_sprite_surface<'a, 'display>(
        &'a self,
        display: &'display glium::Display,
    ) -> anyhow::Result<glium::framebuffer::SimpleFrameBuffer<'a>> {
        let surface = self.buffer.get_sprite_surface(display)?;
        Ok(surface)
    }

    pub fn get_gbuffer_surface<'a, 'display>(
        &'a self,
        display: &'display glium::Display,
    ) -> anyhow::Result<glium::framebuffer::MultiOutputFrameBuffer<'a>> {
        let surface = self.buffer.get_gbuffer_surface(display)?;
        Ok(surface)
    }

    pub fn get_outline_surface<'a, 'display>(
        &'a self,
        display: &'display glium::Display,
    ) -> anyhow::Result<glium::framebuffer::MultiOutputFrameBuffer<'a>> {
        let surface = self.buffer.get_outline_surface(display)?;
        Ok(surface)
    }

    pub fn get_postprocess_surface<'a, 'display>(
        &'a self,
        display: &'display glium::Display,
    ) -> anyhow::Result<glium::framebuffer::SimpleFrameBuffer<'a>> {
        self.buffer.next_postprocess_texture();
        let surface = self.buffer.get_postprocess_surface(display)?;
        Ok(surface)
    }

    pub fn get_last_postprocess_surface<'a, 'display>(
        &'a self,
        display: &'display glium::Display,
    ) -> anyhow::Result<glium::framebuffer::SimpleFrameBuffer<'a>> {
        let surface = self.buffer.get_postprocess_surface(display)?;
        Ok(surface)
    }

    pub fn get_aux_sample<'a, 'display>(&'a self) -> Sampler<'a, glium::Texture2d> {
        self.buffer.get_aux_sampled()
    }

    pub fn get_last_postprocess_sample<'a, 'display>(&'a self) -> Sampler<'a, glium::Texture2d> {
        self.buffer.get_postprocess_sampled()
    }
}
