use std::{marker::PhantomData, ops::RangeBounds, mem::{size_of, MaybeUninit}};
use wgpu::util::DeviceExt;

use crate::{Graphics, BufferSettings, BufferMapping};
use super::BufferMode;


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

// Plain old data type internally used by buffers
pub trait Content: Clone + Copy + Sync + Send + 'static {}
impl<T: Clone + Copy + Sync + Send + 'static> Content for T {}

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Content, const TYPE: u32> {
    buffer: wgpu::Buffer,
    length: usize,
    capacity: usize,
    settings: BufferSettings,
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

impl<T: Content, const TYPE: u32> Buffer<T, TYPE> {
    // Create a buffer using a slice of elements 
    // (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(graphics: &Graphics, slice: &[T], settings: BufferSettings) -> Option<Self> {
        // Return none if we try to make a null buffer
        if slice.is_empty() && !matches!(settings.mode, BufferMode::Resizable) {
            return None;
        } 

        // Convert the array length to byte size
        let stride = size_of::<T>();
        let size = u64::try_from(stride * slice.len()).unwrap();
        
        // If we use persistent mapping we shall align the size
        let size = if settings.mapping.persistent {
            let frac = size as f32 / wgpu::COPY_BUFFER_ALIGNMENT as f32;
            let ceiled = frac.ceil() as u64;
            wgpu::COPY_BUFFER_ALIGNMENT * ceiled
        } else { size };

        // Cast slice to appropriate raw data
        let data = if slice.is_empty() {
            &[]
        } else {
            if settings.mapping.persistent {
                todo!()
            } else {
                unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, size as usize) }
            }
        };

        // Create the buffer usage flags
        let usage = {
            /*
            // Get map read permissions
            let read = wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST;
            let read = if mode.map_read_permission() { read } else { wgpu::BufferUsages::empty() };

            // Get map write permissions
            let write = wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC;
            let write = if mode.map_write_permission() { write } else { wgpu::BufferUsages::empty() };

            // Get mapping permissions
            let mapping = read | write;

            // Get type usage
            mapping | _type | wgpu::BufferUsages::COPY_DST
            */
            wgpu::BufferUsages::from_bits(TYPE).unwrap() | wgpu::BufferUsages::COPY_DST
        };

        // Create buffer description
        let description = wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: settings.mapping.persistent,
        };

        // Create the raw buffer
        let buffer = graphics.device().create_buffer(&description);

        // Write to the buffer
        let queue  = graphics.queue();
        queue.write_buffer(&buffer, 0, data);
        queue.submit([]);
        None
    }
    

    // Create an empty buffer if we can (resizable)
    pub fn empty(graphics: &Graphics, settings: BufferSettings) -> Option<Self> {
        Self::from_slice(graphics, &[], settings)
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

    // Get the buffer settings
    pub fn settings(&self) -> BufferSettings {
        self.settings
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

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Copy the data from another buffer's range into this buffer's range
    pub fn copy_range_from<const OTHER_TYPE: u32>(
        &mut self,
        range: impl RangeBounds<usize>,
        other: &Buffer<T, OTHER_TYPE>,
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
    pub fn as_view_ranged(&self, range: impl RangeBounds<usize>) -> Option<BufferView<T>> {
        todo!()
    }

    // Create a new ranged buffer writer that can read/write from/to the buffer
    pub fn as_view_ranged_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferViewMut<T>> {
        todo!()
    }

    // Create a buffer reader that uses the whole buffer
    pub fn as_view(&self) -> Option<BufferView<T>> {
        todo!()
    }

    // Create a buffer writer that uses the whole buffer
    pub fn as_mut_view(&mut self) -> Option<BufferViewMut<T>> {
        todo!()
    }
    */
}