use crate::utils::{AccessType, BufferHints};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use opengl::types::GLubyte;
use std::{ffi::c_void, marker::PhantomData, mem::{size_of, ManuallyDrop, MaybeUninit}, ptr::null, ops::Range, alloc::Layout};

// The current location (GPU-CPU) of the buffer
pub enum Location<T> {
    Server(GLuint),
    Client(Vec<T>)
}

// Storage that contains a contiguous array of a specific value on the GPU using an OpenGL buffer
// Buffers can be created on any thread, but the moment you upload them to the GPU, you must access them only from the main thread
pub struct Buffer<T: Copy> {
    // Buffer info
    buffer: Location<T>,    
    target: GLuint,

    // How we will access this buffer on the GPU/CPU
    hints: BufferHints,
}

impl<T: Copy> Buffer<T> {
    // Create a buffer from an already existing vector
    pub fn from_vec(target: GLuint, hints: BufferHints, vec: Vec<T>) -> Self {
        Self {
            target,
            buffer: Location::Client(vec),
            hints,
        }
    }

    // Create a new empty buffer that can be temporarily be stored on any thread
    pub fn new(target: GLuint, hints: BufferHints) -> Self {
        Self {
            target,
            buffer: Location::Client(Vec::new()),
            hints,
        }
    } 


    // Upload the buffer to the GPU (PS: This must be called on the current OpenGL context thread)
    // TODO: Context shit
    pub fn upload(&mut self) -> Option<()> {
        if let Location::Client(vec) = &mut self.buffer {
            // Take the vector and send it to the GPU, basically
            let vec = std::mem::take(vec);

            // Create the GPU buffer
            unsafe {
                let mut buffer = 0;
                gl::GenBuffers(1, &mut buffer);
                
                // Initialize it with the data if we can do so
                if vec.capacity() > 0 {
                    // Convert the element-cap to byte-cap
                    let cap = isize::try_from(vec.capacity() * size_of::<T>()).unwrap();
                    // Read from the cached vec since that's where we'd temporarely store the data
                    let ptr = vec.as_ptr();
                    if self.hints.dynamic {
                        // Allocate a region of GPU memory that we can reallocate whenever we want
                        gl::NamedBufferData(buffer, cap, ptr as *const c_void, self.hints.into_access_hints());
                    } else {
                        // Allocate the memory once, basically locking it
                        gl::NamedBufferStorage(buffer, cap, ptr as *const c_void, self.hints.into_mapped_buffer_hints());
                    }
                } 

                // Save the name of the new buffer
                self.buffer = Location::Server(buffer);
            }
            Some(())
        } else {
            // The buffer is already stored on the GPU, nothing to do 
            None
        }
    }

    // Push a new element to the back of the buffer
    pub fn push(&mut self, val: T) {
        match self.buffer {
            Location::Server(buffer) => todo!(),
            Location::Client(vec) => {
                // Check if we can reallocate safely, and panic if we cannot
                let must_reallocate = vec.spare_capacity_mut().len() == 0;
                assert!(!(must_reallocate && !self.hints.dynamic), "Cannot reallocate, buffer is static!");
            },
        }
    }
    // Remove the last element from the buffer
    pub fn pop(&mut self) -> Option<T> {

    }
}

impl<T: Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            match self.buffer {
                Location::Server(mut buffer) => gl::DeleteBuffers(1, &mut buffer),
                _ => {}
            }
        }
    }
}