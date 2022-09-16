use ahash::{AHashMap, AHashSet};
use parking_lot::{Mutex, RwLock};
use std::{
    io::Read,
    marker::PhantomData,
    ops::RangeBounds,
    sync::{
        atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
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
        range: (usize, usize),
    },

    // The buffer data was copied to client memory using glGetBufferSubData
    Copied {
        buf: &'a Buffer<T, TARGET>,
        range: (usize, usize),
        vec: Vec<T>,
    },

    // The buffer data was fecthed from a persistent buffer accessor
    PersistentAccessor {
        ptr: *const T,
        range: (usize, usize),
        accessor: &'a PersistentAccessor<T, TARGET>,
        used: Arc<AtomicBool>,
    },
}

impl<'a, T: Shared, const TARGET: u32> BufferView<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                BufferView::PersistentlyMapped { ptr, range, .. }
                | BufferView::PersistentAccessor { ptr, range, .. } => {
                    std::slice::from_raw_parts(*ptr, range.1 - range.0)
                }
                BufferView::Copied { vec, .. } => vec.as_slice(),
            }
        }
    }

    // Get the range indices that we fetched from the buffer
    pub fn range(&self) -> (usize, usize) {
        match self {
            BufferView::PersistentlyMapped { range, .. }
            | BufferView::Copied { range, .. }
            | BufferView::PersistentAccessor { range, .. } => *range,
        }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for BufferView<'a, T, TARGET> {
    fn drop(&mut self) {
        match self {
            BufferView::PersistentlyMapped { buf, .. } => unsafe {
                gl::UnmapNamedBuffer(buf.name());
            },
            BufferView::PersistentAccessor { used, .. } => {
                used.store(false, Ordering::Relaxed)
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
        range: (usize, usize),
    },

    // The buffer was temporarily cloned to client memory for modification
    Copied {
        buf: &'a mut Buffer<T, TARGET>,
        vec: Vec<T>,
        range: (usize, usize),
    },

    // The buffer data was fecthed from a persistent buffer accessor
    PersistentAccessor {
        ptr: *mut T,
        range: (usize, usize),
        accessor: &'a mut PersistentAccessor<T, TARGET>,
        used: Arc<AtomicBool>,
    },
}

impl<'a, T: Shared, const TARGET: u32> BufferViewMut<'a, T, TARGET> {
    // Get an immutable slice that we can read from
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            match self {
                BufferViewMut::Mapped { ptr, range, .. }
                | BufferViewMut::PersistentAccessor { ptr, range, .. } => {
                    std::slice::from_raw_parts(*ptr as *const T, range.1 - range.0)
                }
                BufferViewMut::Copied { vec, .. } => vec.as_slice(),
            }
        }
    }

    // Get a mutabkle slice that we read/write from/to
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            match self {
                BufferViewMut::Mapped { ptr, range, .. }
                | BufferViewMut::PersistentAccessor { ptr, range, .. } => {
                    std::slice::from_raw_parts_mut(*ptr, range.1 - range.0)
                }
                BufferViewMut::Copied { vec, .. } => vec.as_mut_slice(),
            }
        }
    }

    // Get the range indices that we fetched from the buffer
    pub fn range(&self) -> (usize, usize) {
        match self {
            BufferViewMut::Mapped { range, .. }
            | BufferViewMut::PersistentAccessor { range, .. }
            | BufferViewMut::Copied { range, .. } => *range,
        }
    }
}

impl<'a, T: Shared, const TARGET: u32> Drop for BufferViewMut<'a, T, TARGET> {
    fn drop(&mut self) {
        match self {
            BufferViewMut::Mapped { buf, .. } => unsafe {
                gl::UnmapNamedBuffer(buf.name());
            },
            BufferViewMut::Copied { buf, vec, .. } => {
                buf.write(&vec);
            },
            BufferViewMut::PersistentAccessor { used, .. } => {
                used.store(false, Ordering::Relaxed)
            },
            _ => {}
        }
    }
}

// This is a wrapper type that we can use around buffers to hint that they are persistently mapped
// This will allow us to share the persistent buffer accross threads to be able to read/write from it concurrently
pub struct Persistent<T: Shared, const TARGET: u32> {
    pub(super) buf: Option<Buffer<T, TARGET>>,
    pub(super) ptr: *mut T,
    pub(super) used: Arc<AtomicBool>,
}

// This will be able to read / write to specific parts to a persistent buffer in another thread
// TODO: SLightly rename this to be more correct, or rewrite the persistent buffer API completely
pub struct PersistentAccessor<T: Shared, const TARGET: u32> {
    pub(super) buf: u32,
    pub(super) len: usize,
    pub(super) used: Arc<AtomicBool>,
    pub(super) ptr: *mut T,
}
unsafe impl<T: Shared, const TARGET: u32> Send for PersistentAccessor<T, TARGET> {}
unsafe impl<T: Shared, const TARGET: u32> Sync for PersistentAccessor<T, TARGET> {}

impl<T: Shared, const TARGET: u32> Persistent<T, TARGET> {
    // Unmap the buffer, and return it's underlying buffer value
    // This will return None if the buffer is in use currently in another thread
    pub fn unmap(mut self) -> Option<Buffer<T, TARGET>> {
        if self.used.load(Ordering::Relaxed) {
           return None 
        }

        let buf = self.buf.take().unwrap();
        unsafe {
            gl::UnmapNamedBuffer(buf.name());
        }
        Some(buf)
    }

    /*
    // Get the underlying buffer immutably
    pub fn data(&self) -> &Buffer<T, TARGET> {
        &self.buf
    }

    // Get the underlying buffer mutably
    pub fn data_mut(&mut self) -> &mut Buffer<T, TARGET> {
        &mut self.buf
    }
    */
}

impl<T: Shared, const TARGET: u32> PersistentAccessor<T, TARGET> {
    // Create an immutable buffer view that we can use to read from the persistent buffer's subregion
    pub fn as_view(&self) -> BufferView<T, TARGET> {
        self.used.store(true, Ordering::Relaxed);
        BufferView::PersistentAccessor {
            ptr: self.ptr,
            range: (0, self.len),
            used: self.used.clone(),
            accessor: self,
        }
    }

    // Create a mutable buffer view that we can use to read/write from/to the persistent buffer's subregion
    pub fn as_mut_view(&mut self) -> BufferViewMut<T, TARGET> {
        self.used.store(true, Ordering::Relaxed);
        BufferViewMut::PersistentAccessor {
            ptr: self.ptr,
            range: (0, self.len),
            used: self.used.clone(),
            accessor: self,
        }
    }
}

impl<T: Shared, const TARGET: u32> Drop for Persistent<T, TARGET> {
    fn drop(&mut self) {
        if let Some(buf) = &self.buf {
            let used = self.used.load(Ordering::Relaxed);
            assert!(!used, "Cannot unmap the buffer, it is still in use in another thread");
            
            unsafe {
                gl::UnmapNamedBuffer(buf.name());
            }
        }
    }
}
