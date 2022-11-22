use std::{marker::PhantomData, ops::RangeBounds};
use crate::Graphics;

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Static buffers are only created once, and they can never be modified ever again
    Static,

    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    #[default]
    Resizable,
}

// Bitmask from WGPU BufferUsages
const VERTEX: u32 = wgpu::BufferUsages::VERTEX.bits();
const INDEX: u32 = wgpu::BufferUsages::INDEX.bits();
const STORAGE: u32 = wgpu::BufferUsages::STORAGE.bits();
const UNIFORM: u32 = wgpu::BufferUsages::UNIFORM.bits();
const INDIRECT: u32 = wgpu::BufferUsages::INDIRECT.bits();

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T, const TYPE: u32> {
    buffer: wgpu::Buffer,
    mode: BufferMode,
    _phantom: PhantomData<T>,
}

impl<T, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements 
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(graphics: &Graphics, slice: &[T], mode: BufferMode) -> Option<Self> {
        None
    }

    // Create an empty buffer if we can (resizable)
    pub fn empty(graphics: &Graphics, mode: BufferMode) -> Option<Self> {
        todo!()
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        todo!()
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        todo!()
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        todo!()
    }

    // Convert a range bounds type into the range indices
    // This will return None if the returning indices have a length of 0
    pub fn convert_range_bounds(&self, range: impl RangeBounds<usize>) -> Option<(usize, usize)> {
        todo!()
    }

    // Fills a range in the buffer with a constant value
    pub fn splat_range(&mut self, val: T, range: impl RangeBounds<usize>) {
        todo!()
    }

    // Extent the current buffer using data from an iterator
    pub fn extend_from_iterator<I: Iterator<Item = T>>(&mut self, iterator: I) {
        todo!()
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        todo!()
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(&mut self, slice: &[T], range: impl RangeBounds<usize>) {
        todo!()
    }

    // Read a region of the buffer into a mutable slice
    pub fn read_range(&self, slice: &mut [T], range: impl RangeBounds<usize>) {
        todo!()
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(&self, range: impl RangeBounds<usize> + Copy) -> Vec<T> {
        todo!()
    }

    // Read the whole buffer into a new vector
    pub fn read_to_vec(&self) -> Vec<T> {
        todo!()
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        todo!()
    }

    /*
    // Get an untyped buffer reference of the current buffer
    pub fn untyped_format(&self) -> UntypedBufferFormat {
        todo!()
    }
    */

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        todo!()
    }

    // Copy the data from another buffer's range into this buffer's range
    pub fn copy_range_from<const OTHER: u32>(
        &mut self,
        range: impl RangeBounds<usize>,
        other: &Buffer<T, OTHER>,
        offset: usize,
    ) {
        todo!()
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(&mut self, other: &Buffer<T, OTHER>) {
        todo!()
    }

    // Fills the whole buffer with a constant value
    pub fn splat(&mut self, val: T) {
        todo!()
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        todo!()
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&self, slice: &mut [T]) {
        todo!()
    }

    /*
    // Create a new ranged buffer reader that can read from the buffer
    pub fn as_view_ranged(&self, range: impl RangeBounds<usize>) -> Option<BufferView<T, TARGET>> {
        todo!()
    }

    // Create a new ranged buffer writer that can read/write from/to the buffer
    pub fn as_view_ranged_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferViewMut<T, TARGET>> {
    }

    // Create a buffer reader that uses the whole buffer
    pub fn as_view(&self) -> Option<BufferView<T, TARGET>> {
    }

    // Create a buffer writer that uses the whole buffer
    pub fn as_mut_view(&mut self) -> Option<BufferViewMut<T, TARGET>> {
    }
    */
}