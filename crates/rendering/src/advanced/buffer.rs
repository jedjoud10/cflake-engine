use crate::utils::{AccessType, BufferHints};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use opengl::types::GLubyte;
use std::{ffi::c_void, marker::PhantomData, mem::{size_of, ManuallyDrop, MaybeUninit}, ptr::{null, NonNull}, ops::Range, alloc::Layout};

// Storage that contains a contiguous array of a specific value on the GPU using an OpenGL buffer
// This must always be created on the OpenGL context thread
pub struct Buffer<T: Copy> {
    // Buffer info
    buffer: GLuint,
    target: GLuint,

    // How we will access this buffer on the GPU/CPU
    hints: BufferHints,

    // Allocation
    length: usize,
    capacity: usize,

    // Boo
    _phantom: PhantomData<*const T>,
}

impl<T: Copy> Buffer<T> {
    // Create the buffer from it's raw parts
    pub unsafe fn from_raw_parts(target: GLuint, hints: BufferHints, length: usize, capacity: usize, ptr: *const T) -> Self {
        // Create the buffer
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(target, buffer);

        // Initialize the buffer if we can
        if capacity > 0 {
            // If the pointer is danling, reset it to the null pointer
            let ptr = if ptr == NonNull::<T>::dangling().as_ptr() {
                null()
            } else { ptr };

            // Transform
            let cap = isize::try_from(capacity * size_of::<T>()).unwrap();
            
            // Initialize le data
            if hints.dynamic {
                // Initialize mutable storage
                gl::NamedBufferData(buffer, cap, ptr as *const _, hints.into_mutable_buffer_hints());
            } else {
                // Initialize immutable storage
                gl::NamedBufferStorage(buffer, cap, ptr as *const _, hints.into_immutable_storage_hints());
            }
        }

        Self {
            target,
            buffer,
            hints,
            length,
            capacity,
            _phantom: Default::default(),
        }
    }

    // Create the buffer from a vector that might be valid
    pub fn from_vec(target: GLuint, hints: BufferHints, vec: Vec<T>) -> Self {
        unsafe { Self::from_raw_parts(target, hints, vec.len(), vec.capacity(), vec.as_ptr()) }
    }

    // Create a new empty buffer that can be temporarily be stored on any thread
    // TODO: Context
    pub fn new(target: GLuint, hints: BufferHints) -> Self {
        unsafe { Self::from_raw_parts(target, hints, 0, 0, null()) }
    }

    // Map the buffer for reading
    fn read(&self, offset: usize, length: usize) -> Read<T> {
        // Validate the indices
    }

    // Map the buffer for writing
    fn write(&mut self, offset: usize, length: usize) -> Write<T> {
        // Validate the indices
    }
}

// Read wrapper that'll let us read data from the buffer
pub struct Read<'a, T: Copy> {
    // The buffer
    buffer: &'a Buffer<T>,

    // The mapped pointer given by OpenGL
    ptr: *const T,
}

// Write wrapper that'll let us write data to the buffer
pub struct Write<'a, T: Copy> {
    // The buffer
    buffer: &'a mut Buffer<T>,

    // The mapped pointer given by OpenGL
    ptr: *mut T,
}

impl<T: Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer)
        }
    }
}