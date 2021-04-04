use glium::{
    backend::Facade,
    buffer::{Content, WriteMapping},
    vertex::VertexBufferSlice,
    Vertex, VertexBuffer,
};

pub struct VertexCache<T>
where
    T: Vertex + Content + Copy,
{
    buffer: VertexBuffer<T>,
    pub count: usize,
}

impl<T> VertexCache<T>
where
    T: Vertex + Content + Copy,
{
    pub fn new<F: Facade>(facade: &F, max_count: usize) -> Self {
        Self {
            buffer: VertexBuffer::empty_dynamic(facade, max_count).unwrap(),
            count: 0,
        }
    }

    pub fn as_slice(&self) -> VertexBufferSlice<'_, T> {
        let count = self.count;
        self.buffer.slice(..count).unwrap()
    }
}

pub struct VertexWriter<'buffer, T>
where
    [T]: Content,
    T: Vertex + Copy,
{
    index: &'buffer mut usize,
    mapping: WriteMapping<'buffer, [T]>,
}

impl<'buffer, T> VertexWriter<'buffer, T>
where
    [T]: Content,
    T: Vertex + Copy,
{
    pub fn new(cache: &'buffer mut VertexCache<T>) -> Self {
        cache.count = 0;
        let mapping = cache.buffer.map_write();
        Self {
            index: &mut cache.count,
            mapping,
        }
    }

    pub fn write(&mut self, value: T) {
        self.mapping.set(*self.index, value);
        *self.index += 1;
    }
}
