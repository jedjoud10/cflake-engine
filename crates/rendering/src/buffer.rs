use std::{marker::PhantomData, ptr::{NonNull, null}, mem::size_of, ffi::c_void};
use crate::Context;

// Objects that can be sent to the CPU
// TODO: Rename
pub trait GPUSendable: Copy + Sized {}
impl<T: Copy + Sized> GPUSendable for T {}

// An abstraction layer over a valid OpenGL buffer
pub struct Buffer<T: GPUSendable> {
    // OpenGL buffer data
    buffer: u32,
    length: usize,
    capacity: usize,

    _phantom: PhantomData<*const T>,
}

impl<T: GPUSendable> Buffer<T> {
    // Create a new buffer from it's raw parts
    pub unsafe fn from_raw_parts(_ctx: &Context, immutable: bool, length: usize, capacity: usize, ptr: *const T) -> Self {
        // Create the new OpenGL buffer
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);

        // Convert length and capacity to byte length and byte capacity
        //let byte_length = isize::try_from(length * size_of::<T>()).unwrap();
        let byte_capacity = isize::try_from(capacity * size_of::<T>()).unwrap();

        // Validate the pointer
        let ptr = if capacity == 0 {
            null()
        } else { ptr as *const c_void };

        // Cock and balls
        if immutable {
            // Upload immutable data to the GPU. Immutable buffers cannot be reallocated 
            gl::NamedBufferStorage(buffer, byte_capacity, ptr as _, 0);
        } else {
            // Upload mutable data to the GPU. Mutable buffers can be resized and reallocated
            gl::NamedBufferData(buffer, byte_capacity, ptr as _, 0);
        }

        // Create the buffer struct
        Self {
            buffer,
            length,
            capacity,
            _phantom: Default::default(),
        }
    }
}

// An immutable mapped buffer that we can use to read data from the OpenGL buffer
pub struct RefMapped<'a, T: GPUSendable> {
    buf: &'a Buffer<T>
}

impl<'a, T: GPUSendable> RefMapped<'a, T> {
    // Read from the buffer
}

// A mutable mapped buffer that we can use to write/read to/from the OpenGL buffer
pub struct MutMapped<'a, T: GPUSendable> {
    buf: &'a mut Buffer<T>
}

impl<'a, T: GPUSendable> MutMapped<'a, T> {
    // Read from the buffer
    // Write to the buffer
}