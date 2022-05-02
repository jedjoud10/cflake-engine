use crate::utils::{BufferHints};
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
                gl::NamedBufferData(buffer, cap, ptr as *const _, hints.usage_hints());
            } else {
                // Initialize immutable storage
                gl::NamedBufferStorage(buffer, cap, ptr as *const _, hints.mapped_access_bit());
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

    // Get the target gl type
    pub fn target(&self) -> GLuint {
        self.target
    }

    // Get the buffer's gl name
    pub fn name(&self) -> GLuint {
        self.buffer
    }

    // Get a mapped OpenGL pointer
    unsafe fn map(&self, offset: usize, length: usize) -> *mut T {
        assert!(offset + length < self.length, "Indices out of bound");

        // Transform        
        let offset = isize::try_from(offset * size_of::<T>()).unwrap();        
        let length = isize::try_from(length * size_of::<T>()).unwrap();

        // Map the buffer into client space
        gl::MapBufferRange(self.target, offset, length, self.hints.mapped_access_bit()) as *mut T
    }

    // Map the buffer for reading
    pub fn read(&self, offset: usize, length: usize) -> Option<Read<T>> {
        // Make sure we can even read from the data
        self.hints.readable().then(|| unsafe {
            let ptr = self.map(offset, length) as *const T;
            let slice = std::slice::from_raw_parts(ptr, length);
            Read {
                buffer: self,
                ptr,
                range: offset..(offset+length),
                slice,
            }
        })
    }

    // Map the buffer for writing
    pub fn write(&mut self, offset: usize, length: usize) -> Option<Write<T>> {
        // Make sure we can even write to the data
        self.hints.writable().then(|| unsafe {
            let ptr = self.map(offset, length) as *mut T;
            let slice = std::slice::from_raw_parts_mut(ptr, length);
            Write {
                buffer: self,
                ptr,
                range: offset..(offset+length),
                slice,
            }
        })
    }
}

// Read wrapper that'll let us read data from the buffer
pub struct Read<'a, T: Copy> {
    // Rust data
    buffer: &'a Buffer<T>,
    range: Range<usize>,

    // Kinda unsafe but it works
    ptr: *const T,
    slice: &'a [T],
}


// Write wrapper that'll let us write data to the buffer
pub struct Write<'a, T: Copy> {
    // Rust data
    buffer: &'a mut Buffer<T>,
    range: Range<usize>,

    // Indeed
    ptr: *mut T,
    slice: &'a mut [T],
}

impl<T: Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer)
        }
    }
}