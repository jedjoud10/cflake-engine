use parking_lot::{Mutex, RwLock};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    }, io::Read,
};

use super::Buffer;
use crate::context::{Shared, ToGlName};

// This will allow us to read from the buffer as is if were a Rust slice 
// TODO: Research a way to make this more flexible to allow for better performance
pub enum MappedBuffer<'a, T: Shared, const TARGET: u32> {
    // The buffer was mapped persistently to allow us to read from the buffer whilst it is mapped
    PersistentlyMapped {
        buf: &'a Buffer<T, TARGET>,
        ptr: *const T,
        len: u32,
    },

    // The buffer data was copied to client memory using glGetBufferSubData
    Copied {
        buf: &'a Buffer<T, TARGET>,
        vec: Vec<T>,
    },
}


impl<'a, T: Shared, const TARGET: u32> MappedBuffer<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                MappedBuffer::PersistentlyMapped { ptr, len, .. } => std::slice::from_raw_parts(*ptr, *len as usize),
                MappedBuffer::Copied { vec, .. } => vec.as_slice(),
            }
        }
    }
}

// This will allow us to read AND write from/to the buffer as if it were a mutable Rust slice
// TODO: Research a way to make this more flexible to allow for better performance
pub enum MappedBufferMut<'a, T: Shared, const TARGET: u32> {
    // The buffer was mapped normally to allow us to read/write to it
    Mapped {
        buf: &'a mut Buffer<T, TARGET>,
        ptr: *mut T,
        len: u32,
    }, 
}

impl<'a, T: Shared, const TARGET: u32> MappedBufferMut<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                MappedBufferMut::Mapped {  ptr, len, .. } => std::slice::from_raw_parts(*ptr as *const T, *len as usize),
            }
        }
    }

    // Get a mutabkle slice that we read/write from/to
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            match self {
                MappedBufferMut::Mapped { ptr, len, .. } => std::slice::from_raw_parts_mut(*ptr, *len as usize),
            }
        }
    }
}

// This is a wrapper type that we can use around buffers to hint that they are persistently mapped
pub struct Persistent<T: Shared, const TARGET: u32> {
    pub(super) buf: Buffer<T, TARGET>,
    pub(super) ptr: *mut T,
}

impl<T: Shared, const TARGET: u32> Persistent<T, TARGET> {
    // Unmap the buffer, and return it's underlying buffer value
    pub fn unmap(self) -> Buffer<T, TARGET> {
        unsafe {
            gl::UnmapNamedBuffer(self.buf.name());
        }

        self.buf
    }

    // Get the raw pointer that references the mapped buffer's contents
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}