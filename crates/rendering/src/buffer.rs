use crate::object::{ToGlName, ToGlTarget};
use crate::{context::Context, object::Shared};
use std::mem::MaybeUninit;
use std::ops::{Range, RangeBounds};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
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
    // Create a buffer using a slice of elements
    pub fn from_slice(slice: &[T], mode: BufferMode) -> Self {
        todo!()
    }

    // Create an empty buffer. Only used internally
    pub fn empty(mode: BufferMode) -> Self {
        Self::from_slice(&[], mode)
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

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Clear the values specified by the range to a new value
    pub fn splat_range(&mut self, val: T, range: impl RangeBounds<usize>) {
        todo!()
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

    // Read a region of the buffer into a mutable slice
    pub fn read_range(&mut self, slice: &mut [T], range: impl RangeBounds<usize>) {
        todo!()
    }

    // Copy the buffer contents of Self into Other
    pub fn copy_into<U: Shared, const OTHER: u32>(&self, other: &mut Buffer<U, OTHER>) {
        todo!()
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        todo!()
    }

    // Cast the buffer to a buffer of another target / type
    // The type U and T must have the same exact size and alignment
    pub unsafe fn cast<U: Shared, const OTHER: u32>(self) -> Buffer<U, OTHER> {
        Buffer::<U, OTHER> { buffer: self.buffer, length: self.length, capacity: self.capacity, mode: self.mode, _phantom: Default::default(), _phantom2: Default::default() }
    } 

    // Clear the whole contents of the buffer to the specified value 
    pub fn splat(&mut self, val: T) {
        self.splat_range(val, ..)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        self.write_range(slice, ..)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&mut self, slice: &mut [T]) {
        self.read_range(slice, ..)
    }

    // Map the buffer temporarily for reading only
    pub fn map(&self) -> Mapped<T, TARGET> {
        todo!()
    }
    
    // Map the buffer temporarily for writing AND reading
    pub fn map_mut(&mut self) -> MappedMut<T, TARGET> {
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

// Immutably mapped buffer that we read from directly
pub struct Mapped<'a, T: Shared, const TARGET: u32> {
    buffer: &'a Buffer<T, TARGET>,
    ptr: *const T,
}

// Mutably mapped buffer that we can write / read from directly
pub struct MappedMut<'a, T: Shared, const TARGET: u32> {
    buffer: &'a mut Buffer<T, TARGET>,
    ptr: *mut T,
}


impl<'a, T: Shared, const TARGET: u32> Mapped<'a, T, TARGET> {
    // Convert the mapped pointer into an immutable slice
    pub fn as_slice(&self) -> &[T] {
        todo!()
    }
}

impl<'a, T: Shared, const TARGET: u32> MappedMut<'a, T, TARGET> {
    // Convert the mapped buffer into an immutable slice
    pub fn as_slice(&self) -> &[T] {
        todo!()
    }
    
    // Convert the mapped buffer into a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        todo!()        
    }
}