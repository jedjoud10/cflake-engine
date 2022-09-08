use std::{marker::PhantomData, sync::{Arc, atomic::{AtomicU64, Ordering}}};
use parking_lot::{Mutex, RwLock};

use crate::context::{Shared, ToGlName};
use super::Buffer;

// Immutably mapped buffer that we can read from directly
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

// This is a global bitfield that will be shared around threads that wish to use persistently mapped buffers
pub(crate) struct PersistentlyMappedBuffers {
    // A bitfield of buffers and their persistent mapping state
    // This u64 is divided into equal sections of 2 bits each
    // First bit (from the right) of each section is the "persistently mapped" flag
    // Second bit (from the right) of each section is the "mutably mapped" flag
    chunks: Arc<RwLock<Vec<AtomicU64>>>
}

impl Default for PersistentlyMappedBuffers {
    fn default() -> Self {
        Self { chunks: Arc::new(RwLock::new(Vec::new())) }
    }
}

impl Clone for PersistentlyMappedBuffers {
    fn clone(&self) -> Self {
        Self { chunks: self.chunks.clone() }
    }
}

impl PersistentlyMappedBuffers {    
    // Update the state of a persistently mapped buffer
    pub(crate) fn update_mapped_buffer_state(&self, name: u32, mapped: bool, mutable: bool) {
        let read = self.chunks.read();
        let chunk = name / 32;
        let location = (name % 32) * 2; 

        // Extent the chunks if we are missing some
        if chunk as usize >= read.len() {
            drop(read);
            let mut write = self.chunks.write();
            let missing = (chunk as usize - write.len()) + 1;
            write.extend((0..missing).into_iter().map(|_| AtomicU64::new(0)));
        } 

        // Write to the new location
        let read = self.chunks.read();
        let atomic = &read[chunk as usize];
        let mut read = atomic.load(Ordering::Relaxed);
        
        // Set the mapped bit
        // TODO: This shit ugly pls fix
        if mapped {
            read |= 1 << location;  
        } else {
            read &= !(1 << location);
        }

        // Set the mutation bit
        // TODO: This shit ugly pls fix
        if mutable {
            read |= 3 << location;  
        } else {
            read &= !(3 << location);
        }

        atomic.store(read, Ordering::Relaxed);
    }

    // Get the state of the buffer
    // First boolean tells us if the buffer is used persistently, second boolean tells us if it is used mutably
    pub(crate) fn get_buffer_state(&self, name: u32) -> (bool, bool) {
        let read = self.chunks.read();
        let chunk = name / 32;
        let location = (name % 32) * 2; 
        let atomic = &read[chunk as usize];
        let read = atomic.load(Ordering::Relaxed);
        let mapped = (read >> location) & 1 == 1;
        let mutable = (read >> location) & 3 == 1;
        (mapped, mutable)
    }
}

// Immutably mapped persistent buffer that we can read from directly AND from other threads
pub struct PersistentlyMapped<T: Shared, const TARGET: u32> {
    pub(super) buffer: PhantomData<Buffer<T, TARGET>>,
    pub(super) len: usize,
    pub(super) ptr: *const T,
}

// Mutably mapped persistent buffer that we write / read from in other threads
pub struct PersistentlyMappedMut<T: Shared, const TARGET: u32> {
    pub(super) buffer: PhantomData<Buffer<T, TARGET>>,
    pub(super) len: usize,
    pub(super) ptr: *mut T,
}
