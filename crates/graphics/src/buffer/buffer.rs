use std::{marker::PhantomData, mem::size_of, ops::RangeBounds};

use crate::{Content, Graphics};
use bytemuck::Zeroable;

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers are only created once, but they allow the user to mutate each element
    #[default]
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}

// Bitmask from WGPU BufferUsages
const VERTEX: u32 = 0;
const INDEX: u32 = 1;
const STORAGE: u32 = 2;
const UNIFORM: u32 = 3;
const INDIRECT: u32 = 4;

// Common buffer types
pub type VertexBuffer<T> = Buffer<T, VERTEX>;
pub type IndexBuffer<T> = Buffer<T, INDEX>;
pub type StorageBuffer<T> = Buffer<T, STORAGE>;
pub type UniformBuffer<T> = Buffer<T, UNIFORM>;
pub type IndirectBuffer<T> = Buffer<T, INDIRECT>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Content, const TYPE: u32> {
    length: usize,
    capacity: usize,
    mode: BufferMode,
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

// Internal bounds used by the buffer
pub(super) struct BufferBounds {
    offset: usize,
    size: usize,
}

// Multiple ways of writing to a buffer
// 1. Create a staging buffer, writing to it, then copy
// 2. Create a staging buffer, mapping it and writing to it, then copy
// 3. Write to buffer directly

// Multiple ways of reading from a buffer
// 1. Create a staging buffer as copy dst, copy, then map

impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(
        _graphics: &Graphics,
        _slice: &[T],
        _mode: BufferMode,
    ) -> Option<Self> {
        None
    }

    // Create an empty buffer if we can (resizable)
    pub fn empty(
        graphics: &Graphics,
        mode: BufferMode,
    ) -> Option<Self> {
        Self::from_slice(graphics, &[], mode)
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Get the buffer mode
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Convert a range bounds type into the range indices
    // This will return None if the returning indices have a length of 0
    fn convert_range_bounds(
        &self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferBounds> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(_) => panic!(),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.length,
        };

        let valid_start_index = start < self.length;
        let valid_end_index = end <= self.length && end >= start;

        if !valid_start_index || !valid_end_index {
            return None;
        }

        if (end - start) == 0 {
            return None;
        }

        Some(BufferBounds {
            offset: start,
            size: end - start,
        })
    }

    // Fills a range in the buffer with a constant value
    pub fn splat_range(
        &mut self,
        _val: T,
        range: impl RangeBounds<usize>,
    ) {
        if let Some(BufferBounds { offset: _, size: _ }) =
            self.convert_range_bounds(range)
        {}
    }

    // Extent the current buffer using data from an iterator
    pub fn extend_from_iterator<I: Iterator<Item = T>>(
        &mut self,
        iterator: I,
    ) {
        let collected = iterator.collect::<Vec<_>>();
        self.extend_from_slice(&collected);
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        assert!(
            matches!(self.mode, BufferMode::Resizable)
                | matches!(self.mode, BufferMode::Parital),
            "Cannot extend buffer, missing permission"
        );

        // Can't do nothing
        if slice.is_empty() {
            return;
        }

        // Allocate the buffer for the first time
        if self.length == 0 && self.capacity == 0 {
            // TODO: Create the buffer
        } else if slice.len() + self.length > self.capacity {
            // create new buffer
            // cpy self -> new buffer
            // update self buffer
        } else {
            // TODO: write to current buffer
        }
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(
        &mut self,
        slice: &[T],
        range: impl RangeBounds<usize>,
    ) {
        let Some(BufferBounds {
            offset: _, size 
        }) = self.convert_range_bounds(range) else {
            return;
        };
        assert_eq!(
            size,
            slice.len(),
            "Buffer write range is not equal to slice length"
        );

        // TODO: write to current buffer
    }

    // Read a region of the buffer into a mutable slice immediately
    pub fn read_range(
        &self,
        _slice: &mut [T],
        range: impl RangeBounds<usize>,
    ) {
        let Some(BufferBounds {
            offset: _, size: _ 
        }) = self.convert_range_bounds(range) else {
            return;
        };

        // TODO: read from current buffer
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(
        &self,
        range: impl RangeBounds<usize> + Copy,
    ) -> Vec<T> {
        let Some(BufferBounds {
            size, .. 
        }) = self.convert_range_bounds(range) else {
            return Vec::new();
        };

        // Create a vec and read into it
        let mut vec = vec![T::zeroed(); size];
        self.read_range(&mut vec, range);
        vec
    }

    // Read the whole buffer into a new vector
    pub fn read_to_vec(&self) -> Vec<T> {
        self.read_range_as_vec(..)
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        // TODO: write to current buffer
        self.length = 0;
    }

    // Copy the data from another buffer's range into this buffer's range
    // dst_offset refers to the offset inside Self
    // src_range refers to the range of 'other'
    pub fn copy_range_from<const OTHER_TYPE: u32>(
        &mut self,
        _src_range: impl RangeBounds<usize>,
        _other: &Buffer<T, OTHER_TYPE>,
        _dst_offset: usize,
    ) {
        todo!()
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(
        &mut self,
        other: &Buffer<T, OTHER>,
    ) {
        assert_eq!(
            self.len(),
            other.len(),
            "Cannot copy from buffer, length mismatch"
        );

        self.copy_range_from(.., other, 0);
    }

    // Fills the whole buffer with a constant value
    pub fn splat(&mut self, val: T) {
        self.splat_range(val, ..)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        self.write_range(slice, ..)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&self, slice: &mut [T]) {
        self.read_range(slice, ..)
    }

    /*
    // Create a new ranged buffer reader that can read from the buffer
    pub fn as_view_ranged(&self, range: impl RangeBounds<usize>) -> Option<BufferView<T, TYPE>> {
        todo!()
    }

    // Create a new ranged buffer writer that can read/write from/to the buffer
    pub fn as_view_ranged_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferViewMut<T, TYPE>> {
        todo!()
    }

    // Create a buffer reader that uses the whole buffer
    pub fn as_view(&self) -> Option<BufferView<T, TYPE>> {
        todo!()
    }

    // Create a buffer writer that uses the whole buffer
    pub fn as_mut_view(&mut self) -> Option<BufferViewMut<T, TYPE>> {
        todo!()
    }
    */
}
