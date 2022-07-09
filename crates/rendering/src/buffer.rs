use crate::object::{ToGlName, ToGlTarget};
use crate::{context::Context, object::Shared};
use std::mem::MaybeUninit;
use std::ops::{Range, RangeBounds};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BufferMode {
    // Static buffers are only created once, and they can never be modified ever again
    Static,

    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length 
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}

impl BufferMode {
    // Can we read from an arbitrary buffer that uses this buffer mode?
    pub fn read_permission(&self) -> bool {
        true
    }
    
    // Can we write to an arbitrary buffer that uses this buffer mode?
    pub fn write_permission(&self) -> bool {
        match self {
            BufferMode::Static => false,
            _ => true,
        }
    }

    // Can we modify the LENGTH of an arbitrary buffer that uses this buffer mode?
    pub fn modify_length_permission(&self) -> bool {
        match self {
            BufferMode::Resizable | BufferMode::Parital => true,
            _ => false
        }
    }

    // Can we reallocate an arbitrary buffer that uses this buffer mode?
    pub fn reallocate_permission(&self) -> bool {
        match self {
            BufferMode::Resizable => true,
            _ => false
        }
    }
}


// Common OpenGL buffer types
pub type ArrayBuffer<T> = Buffer<T, { gl::ARRAY_BUFFER }>;
pub type ElementBuffer<T> = Buffer<T, { gl::ELEMENT_ARRAY_BUFFER }>;
pub type AtomicBuffer<T> = Buffer<T, { gl::ATOMIC_COUNTER_BUFFER }>;
pub type ComputeStorage<T> = Buffer<T, { gl::SHADER_STORAGE_BUFFER }>;
pub type UniformBuffer<T> = Buffer<T, { gl::UNIFORM_BUFFER }>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Shared, const TARGET: u32> {
    buffer: u32,
    length: usize,
    capacity: usize,
    mode: BufferMode,

    _phantom: PhantomData<*const MaybeUninit<T>>,
    _phantom2: PhantomData<T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a static buffer using a slice of elements
    pub fn static_from_slice(slice: &[T]) -> Option<Self> {
        todo!()
    }

    // Create a dynamic buffer using a slice of elements
    pub fn dynamic_from_slice(slice: &[T]) -> Option<Self> {
        todo!()
    }
    
    // Create a partial buffer using a slice of elements
    pub fn partial_from_slice(slice: &[T]) -> Option<Self> {
        todo!()
    }

    // Create a partial buffer using a specific capacity
    pub fn partial_with_capacity(cap: usize) -> Option<Self> {
        todo!()
    }
    
    // Create a resizable buffer using a slice of elements
    pub fn resizable_from_slice(slice: &[T]) -> Self {
        todo!()
    }
    
    // Create a resizable buffer using a specific capacity
    pub fn resizable_with_capacity(cap: usize) -> Self {
        todo!()
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Clear the values specified by the range to a new value
    pub fn clear_range(&mut self, val: T, range: impl RangeBounds<usize>) {
        todo!()
    }

    // Clear the whole contents of the buffer to the specified value 
    pub fn clear(&mut self, val: T) {
        self.clear_range(val, ..)
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        todo!()
    }

    // Push a single element into the buffer (slow!)
    pub fn push(&mut self, value: T) {
        todo!()
    }

    // Remove the last element from the buffer (slow!)
    pub fn pop(&mut self) {
        todo!()
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(&mut self, slice: &[T], range: impl RangeBounds<usize>) {
        todo!()
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        self.write_range(slice, ..)
    }

    // Read a region of the buffer into a mutable slice
    pub fn read_range(&mut self, slice: &mut [T], range: impl RangeBounds<usize>) {
        todo!()
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&mut self, slice: &mut [T]) {
        self.read_range(slice, ..)
    }

    // Copy the buffer contents of Self into Other
    pub fn copy_into<U: Shared, const OTHER: u32>(&self, other: &mut Buffer<U, OTHER>) {
        todo!()
    }
}

impl<T: Shared, const TARGET: u32> ToGlName for Buffer<T, TARGET> {
    fn name(&self) -> u32 {
        self.buffer
    }
}

impl<T: Shared, const TARGET: u32> ToGlTarget for Buffer<T, TARGET> {
    fn target() -> u32 {
        TARGET
    }
}

impl<T: Shared, const TARGET: u32> Drop for Buffer<T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}