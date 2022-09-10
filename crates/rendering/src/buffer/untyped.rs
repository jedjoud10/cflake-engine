use super::{Buffer, BufferMode};
use crate::context::{Shared, ToGlName};
use std::{any::TypeId, mem::size_of};

// This is an untyped reference to the format of a specific buffer
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UntypedBufferFormat<'a> {
    pub(super) target: u32,
    pub(super) buffer: &'a u32,
    pub(super) length: &'a usize,
    pub(super) capacity: &'a usize,
    pub(super) mode: &'a BufferMode,
    pub(super) _type: TypeId,
    pub(super) stride: usize,
}

impl<'a> UntypedBufferFormat<'a> {
    // Get the OpenGL name of the buffer
    pub fn name(&self) -> u32 {
        *self.buffer
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        *self.length
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        *self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        *self.capacity
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        *self.mode
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        self.stride
    }

    // Get the untyped T type ID
    pub fn type_id(&self) -> TypeId {
        self._type
    }

    // Get the untyped target
    pub fn target(&self) -> u32 {
        self.target
    }
}
