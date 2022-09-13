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
            }
            _ => {}
        }
    }
}

// This is a wrapper type that we can use around buffers to hint that they are persistently mapped
// This will allow us to share the persistent buffer accross threads to be able to read/write from it concurrently
pub struct Persistent<T: Shared, const TARGET: u32> {
    pub(super) buf: Option<Buffer<T, TARGET>>,
    pub(super) ptr: *mut T,
    pub(super) ranges: Vec<(usize, usize)>,
    pub(super) counter: u32,
    pub(super) active: Arc<AtomicBool>,
}

// This will be able to read / write to specific parts to a persistent buffer using multiple threads
// TODO: SLightly rename this to be more correct, or rewrite the persistent buffer API completely
// TODO: Handle cases when the persistent buffer gets deallocated mmoent
pub struct PersistentAccessor<T: Shared, const TARGET: u32> {
    buf: u32,
    idx: u32,
    ptr: *mut T,
    range: (usize, usize),
}
unsafe impl<T: Shared, const TARGET: u32> Send for PersistentAccessor<T, TARGET> {}
unsafe impl<T: Shared, const TARGET: u32> Sync for PersistentAccessor<T, TARGET> {}

impl<T: Shared, const TARGET: u32> Persistent<T, TARGET> {
    // Unmap the buffer, and return it's underlying buffer value
    pub fn unmap(mut self) -> Buffer<T, TARGET> {
        let buf = self.buf.take().unwrap();
        unsafe {
            gl::UnmapNamedBuffer(buf.name());
        }
        buf
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

    // Check if a specific is already in use by a shared viewer
    fn is_range_in_use(&self, range: Option<(usize, usize)>) -> bool {
        if let Some((start1, end1)) = range {
            self.ranges
                .iter()
                .any(|&(start2, end2)| start2 <= end1 && end2 >= start1)
        } else {
            return false;
        }
    }

    // Create a new range that we can read / write to from another thread
    // This function will return None if the range was already used persistently somewhere else
    pub fn accessor_range(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<PersistentAccessor<T, TARGET>> {
        let range = self.buf.as_ref().unwrap().convert_range_bounds(range);
        let range = if !self.is_range_in_use(range) {
            range.unwrap()
        } else {
            return None;
        };

        self.counter += 1;
        Some(PersistentAccessor {
            buf: self.buf.as_ref().unwrap().name(),
            idx: self.counter - 1,
            ptr: self.ptr,
            range,
        })
    }

    // Create a persistent accessor that will span over the whole buffer
    pub fn accessor(&mut self) -> Option<PersistentAccessor<T, TARGET>> {
        self.accessor_range(..)
    }
}

impl<T: Shared, const TARGET: u32> PersistentAccessor<T, TARGET> {
    // Get the range indices that we fetched from the buffer
    pub fn range(&self) -> (usize, usize) {
        self.range
    }

    // Create an immutable buffer view that we can use to read from the persistent buffer's subregion
    pub fn as_view(&self) -> BufferView<T, TARGET> {
        BufferView::PersistentAccessor {
            ptr: self.ptr,
            range: self.range,
            accessor: self,
        }
    }

    // Create a mutable buffer view that we can use to read/write from/to the persistent buffer's subregion
    pub fn as_mut_view(&mut self) -> BufferViewMut<T, TARGET> {
        BufferViewMut::PersistentAccessor {
            ptr: self.ptr,
            range: self.range,
            accessor: self,
        }
    }
}

impl<T: Shared, const TARGET: u32> Drop for Persistent<T, TARGET> {
    fn drop(&mut self) {
        if let Some(buf) = &self.buf {
            unsafe {
                gl::UnmapNamedBuffer(buf.name());
            }
            self.active.fetch_and(false, Ordering::Relaxed);
        }
    }
}
