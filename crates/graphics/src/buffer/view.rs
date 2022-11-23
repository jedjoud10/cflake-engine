use crate::{Content, Buffer};

// This will allow us to read from the buffer as if it was a slice
// This will internally create a staging buffer for reading and map it
pub struct BufferView<'a, T: Content, const TYPE: u32> {
    buffer: &'a Buffer<T, TYPE>,
    staging: wgpu::Buffer,
    range: super::BufferBounds,
}

// This will allow us to read/write from/to the buffer as if it was a slice
// This will internally create a staging buffer for reading and map it
pub struct BufferViewMut<'a, T: Content, const TYPE: u32> {
    buffer: &'a Buffer<T, TYPE>,
    staging: wgpu::Buffer,
    range: super::BufferBounds,
}