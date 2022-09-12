use parking_lot::{Mutex, RwLock};
use std::{
    io::Read,
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use super::Buffer;
use crate::context::{Shared, ToGlName};

// This will allow us to read from the buffer as is if were a Rust slice
// TODO: Research a way to make this more flexible to allow for better performance
pub enum BufferView<'a, T: Shared, const TARGET: u32> {
    // The buffer was mapped persistently to allow us to read from the buffer whilst it is mapped
    PersistentlyMapped {
        buf: &'a Buffer<T, TARGET>,
        ptr: *const T,
        len: usize,
    },

    // The buffer data was copied to client memory using glGetBufferSubData
    Copied {
        buf: &'a Buffer<T, TARGET>,
        vec: Vec<T>,
    },
}

impl<'a, T: Shared, const TARGET: u32> BufferView<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                BufferView::PersistentlyMapped { ptr, len, .. } => {
                    std::slice::from_raw_parts(*ptr, *len)
                }
                BufferView::Copied { vec, .. } => vec.as_slice(),
            }
        }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for BufferView<'a, T, TARGET> {
    fn drop(&mut self) {
        match self {
            BufferView::PersistentlyMapped { buf, .. } => unsafe {
                gl::UnmapNamedBuffer(buf.name());
            },
            _ => {}
        }
    }
}

// This will allow us to read AND write from/to the buffer as if it were a mutable Rust slice
// TODO: Research a way to make this more flexible to allow for better performance
pub enum BufferViewMut<'a, T: Shared, const TARGET: u32> {
    // The buffer was mapped normally to allow us to read/write to it
    Mapped {
        buf: &'a mut Buffer<T, TARGET>,
        ptr: *mut T,
        len: usize,
    },

    // The buffer was temporarily cloned to client memory for modification
    Copied {
        buf: &'a mut Buffer<T, TARGET>,
        vec: Vec<T>,
    },
}

impl<'a, T: Shared, const TARGET: u32> BufferViewMut<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                BufferViewMut::Mapped { ptr, len, .. } => {
                    std::slice::from_raw_parts(*ptr as *const T, *len)
                }
                BufferViewMut::Copied { vec, .. } => vec.as_slice(),
            }
        }
    }

    // Get a mutabkle slice that we read/write from/to
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            match self {
                BufferViewMut::Mapped { ptr, len, .. } => {
                    std::slice::from_raw_parts_mut(*ptr, *len)
                }
                BufferViewMut::Copied { vec, .. } => vec.as_mut_slice(),
            }
        }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for BufferViewMut<'a, T, TARGET> {
    fn drop(&mut self) {
        match self {
            BufferViewMut::Mapped { buf, .. } => unsafe {
                gl::UnmapNamedBuffer(buf.name());
            },
            BufferViewMut::Copied { buf, vec } => {
                buf.write(&vec);
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

    // Get the underlying buffer immutably
    pub fn data(&self) -> &Buffer<T, TARGET> {
        &self.buf
    }

    // Get the underlying buffer mutably
    pub fn data_mut(&mut self) -> &mut Buffer<T, TARGET> {
        &mut self.buf
    }

    // Get the raw pointer that references the mapped buffer's contents
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}
