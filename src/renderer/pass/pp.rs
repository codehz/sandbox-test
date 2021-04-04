use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct PostProcessVertex {
    id: u32,
}

implement_vertex!(PostProcessVertex, id);

impl PostProcessVertex {
    fn get() -> &'static [PostProcessVertex; 4] {
        &[
            PostProcessVertex { id: 0 },
            PostProcessVertex { id: 1 },
            PostProcessVertex { id: 2 },
            PostProcessVertex { id: 3 },
        ]
    }

    fn get_buffer(
        display: &glium::Display,
    ) -> Result<glium::VertexBuffer<PostProcessVertex>, glium::vertex::BufferCreationError> {
        glium::VertexBuffer::new(display, PostProcessVertex::get())
    }
}

#[repr(transparent)]
pub struct PostProcessPass(glium::VertexBuffer<PostProcessVertex>);

impl PostProcessPass {
    pub fn new(display: &glium::Display) -> Result<Self, glium::vertex::BufferCreationError> {
        Ok(Self(PostProcessVertex::get_buffer(display)?))
    }
}

impl<'pass>
    Into<(
        glium::vertex::VertexBufferSlice<'pass, PostProcessVertex>,
        &'static glium::index::NoIndices,
    )> for &'pass PostProcessPass
{
    fn into(
        self,
    ) -> (
        glium::vertex::VertexBufferSlice<'pass, PostProcessVertex>,
        &'static glium::index::NoIndices,
    ) {
        (
            self.0.slice(..).unwrap(),
            &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
        )
    }
}
