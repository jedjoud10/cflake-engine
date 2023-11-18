use crate::{BufferMode, BufferUsage, BufferVariant};

// Untyped buffer that does not contain a generic type nor type ID
pub struct BufferInfo<'a> {
    pub(super) buffer: &'a wgpu::Buffer,
    pub(super) variant: BufferVariant,
    pub(super) length: usize,
    pub(super) stride: usize,
    pub(super) capacity: usize,
    pub(super) usage: BufferUsage,
    pub(super) mode: BufferMode,
}

impl<'a> BufferInfo<'a> {
    // Get the inner raw WGPU buffer
    pub fn raw(&self) -> &'a wgpu::Buffer {
        self.buffer
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length.try_into().unwrap()
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity.try_into().unwrap()
    }

    // Get the buffer usage
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    // Get the buffer mode
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        self.stride
    }

    // Get the buffer variant type
    pub fn variant(&self) -> BufferVariant {
        self.variant
    }
}
