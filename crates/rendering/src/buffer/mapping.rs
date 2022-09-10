use std::{marker::PhantomData, sync::{Arc, atomic::{AtomicU64, Ordering}}};
use parking_lot::{Mutex, RwLock};

use crate::context::{Shared, ToGlName};
use super::Buffer;

// Immutably mapped buffer that we can read from directly
// The reason this takes a mutable reference to the buffer is because OpenGL does not allow us to read the buffer even when it is mapped
pub struct Mapped<'a, T: Shared, const TARGET: u32> {
    pub(super) buffer: &'a Buffer<T, TARGET>,
    pub(super) len: usize,
    pub(super) ptr: *const T,
}

// Mutably mapped buffer that we can write / read from directly
pub struct MappedMut<'a, T: Shared, const TARGET: u32> {
    pub(super) buffer: &'a mut Buffer<T, TARGET>,
    pub(super) len: usize,
    pub(super) ptr: *mut T,
}

impl<'a, T: Shared, const TARGET: u32> Mapped<'a, T, TARGET> {
    // Get the length of the mapped region
    pub fn len(&self) -> usize {
        self.len
    }

    // Convert the mapped pointer into an immutable slice
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for Mapped<'a, T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.buffer.name());
        }
    }
}

impl<'a, T: Shared, const TARGET: u32> MappedMut<'a, T, TARGET> {
    // Get the length of the mapped region
    pub fn len(&self) -> usize {
        self.len
    }

    // Convert the mapped buffer into an immutable slice
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    // Convert the mapped buffer into a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for MappedMut<'a, T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.buffer.name());
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
