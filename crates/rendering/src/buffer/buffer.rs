use super::{BufferView, BufferViewMut, Persistent, UntypedBufferFormat, PersistentAccessor};
use crate::context::{Context, Shared, ToGlName, ToGlTarget};
use std::alloc::Layout;
use std::any::TypeId;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::RangeBounds;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

// Some settings that tell us how exactly we should create the buffer
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Static buffers are only created once, and they can never be modified ever again
    Static {
        map_read: bool,
        persistent: bool,
        client: bool,
    },

    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic {
        map_write: bool,
        map_read: bool,
        persistent: bool,
        client: bool,
    },

    // Partial buffer have a fixed capacity, but a dynamic length
    Parital {
        map_write: bool,
        map_read: bool,
        persistent: bool,
        client: bool,
    },

    // Resizable buffers can be resized to whatever length needed
    #[default]
    Resizable,
}

impl BufferMode {
    // Can we read from an arbitrary buffer that uses this buffer mode?
    pub fn read_permission(&self) -> bool {
        true
    }

    // Can we write to an arbitrary buffer that uses this buffer mode?
    pub fn write_permission(&self) -> bool {
        match self {
            BufferMode::Static { .. } => false,
            _ => true,
        }
    }

    // Can we modify the LENGTH of an arbitrary buffer that uses this buffer mode?
    pub fn modify_length_permission(&self) -> bool {
        match self {
            BufferMode::Resizable | BufferMode::Parital { .. } => true,
            _ => false,
        }
    }

    // Can we reallocate an arbitrary buffer that uses this buffer mode?
    pub fn reallocate_permission(&self) -> bool {
        match self {
            BufferMode::Resizable => true,
            _ => false,
        }
    }

    // Check if we can map the buffer persistently
    pub fn map_persistent_permission(&self) -> bool {
        match self {
            BufferMode::Static { persistent, .. }
            | BufferMode::Dynamic { persistent, .. }
            | BufferMode::Parital { persistent, .. } => *persistent,
            BufferMode::Resizable => false,
        }
    }

    // Check if we can map the buffer for reading
    pub fn map_read_permission(&self) -> bool {
        match self {
            BufferMode::Static { map_read, .. }
            | BufferMode::Dynamic { map_read, .. }
            | BufferMode::Parital { map_read, .. } => *map_read,
            BufferMode::Resizable => true,
        }
    }

    // Check if we can map the buffer for writing
    pub fn map_write_permission(&self) -> bool {
        match self {
            BufferMode::Dynamic { map_write, .. } | BufferMode::Parital { map_write, .. } => {
                *map_write
            }
            BufferMode::Resizable => false,
            BufferMode::Static { .. } => false,
        }
    }

    // Convert the mapping permissions to an OpenGL immutable storage flag
    pub fn map_flags(&self) -> Option<u32> {
        match self {
            BufferMode::Static {
                persistent,
                map_read,
                client,
            } => Some({
                let mut flags = 0u32;
                if *persistent {
                    flags |= gl::MAP_PERSISTENT_BIT
                }
                if *map_read {
                    flags |= gl::MAP_READ_BIT
                }
                if *client {
                    flags |= gl::CLIENT_STORAGE_BIT
                }
                flags
            }),
            BufferMode::Dynamic {
                persistent,
                map_write,
                map_read,
                client,
            }
            | BufferMode::Parital {
                persistent,
                map_write,
                map_read,
                client,
            } => Some({
                let mut flags = 0u32;
                if *persistent {
                    flags |= gl::MAP_PERSISTENT_BIT
                }
                if *map_read {
                    flags |= gl::MAP_READ_BIT
                }
                if *map_write {
                    flags |= gl::MAP_WRITE_BIT
                }
                if *client {
                    flags |= gl::CLIENT_STORAGE_BIT
                }
                flags | gl::DYNAMIC_STORAGE_BIT
            }),
            BufferMode::Resizable => None,
        }
    }
}

// Common OpenGL buffer types
pub type ArrayBuffer<T> = Buffer<T, { gl::ARRAY_BUFFER }>;
pub type ElementBuffer<T> = Buffer<T, { gl::ELEMENT_ARRAY_BUFFER }>;
pub type Triangle<T> = [T; 3];
pub type TriangleBuffer<T> = ElementBuffer<Triangle<T>>;
pub type AtomicBuffer<T> = Buffer<T, { gl::ATOMIC_COUNTER_BUFFER }>;
pub type ComputeStorage<T> = Buffer<T, { gl::SHADER_STORAGE_BUFFER }>;
pub type UniformBuffer<T> = Buffer<T, { gl::UNIFORM_BUFFER }>;
pub type DispatchComputerIndirectBuffer = Buffer<(u32, u32, u32), { gl::DISPATCH_INDIRECT_BUFFER }>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Shared, const TARGET: u32> {
    buffer: u32,
    length: usize,
    capacity: usize,
    mode: BufferMode,
    _phantom: PhantomData<*const MaybeUninit<T>>,
    _phantom2: PhantomData<T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a buffer using a slice of elements (will return none if we try to create a zero length Static, Dynamic, or Partial buffer)
    pub fn from_slice(_ctx: &mut Context, slice: &[T], mode: BufferMode) -> Option<Self> {
        unsafe {
            // We cannot handle zero sized types
            if size_of::<T>() == 0 {
                return None;
            }

            // Return none if we are trying to make an empty static / dynamic / partial buffer
            if slice.is_empty() {
                match mode {
                    BufferMode::Static { .. }
                    | BufferMode::Dynamic { .. }
                    | BufferMode::Parital { .. } => return None,
                    _ => {}
                };
            }

            // Create OpenGL buffer and fetch pointer
            let mut buffer = 0;
            gl::CreateBuffers(1, &mut buffer);
            gl::BindBuffer(TARGET, buffer);
            let bytes = (slice.len() * size_of::<T>()) as isize;
            let ptr = if bytes == 0 {
                null()
            } else {
                slice.as_ptr() as *const c_void
            };

            // Create the immutable vs mutable buffers
            match mode.map_flags() {
                Some(flags) => gl::NamedBufferStorage(buffer, bytes, ptr, flags),
                None => gl::NamedBufferData(buffer, bytes, ptr, gl::DYNAMIC_DRAW),
            }

            // Create the buffer object
            Some(Self {
                buffer,
                length: slice.len(),
                capacity: slice.len(),
                mode,
                _phantom: Default::default(),
                _phantom2: Default::default(),
            })
        }
    }

    // Create an empty buffer. Only used internally
    pub fn empty(ctx: &mut Context, mode: BufferMode) -> Option<Self> {
        Self::from_slice(ctx, &[], mode)
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length
    }

    // Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Convert a range bounds type into the range indices
    // This will return None if the returning indices have a length of 0
    pub fn convert_range_bounds(&self, range: impl RangeBounds<usize>) -> Option<(usize, usize)> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(_) => panic!(),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.length,
        };

        let valid_start_index = start < self.length;
        let valid_end_index = end <= self.length && end >= start;

        if !valid_start_index || !valid_end_index {
            return None;
        }

        if (end - start) == 0 {
            return None;
        }

        Some((start, end))
    }

    // Fills a range in the buffer with a constant value
    pub fn splat_range(&mut self, val: T, range: impl RangeBounds<usize>) {
        unsafe {
            assert!(
                self.mode.write_permission(),
                "Cannot write to buffer, missing permission"
            );
            let (start, end) = self
                .convert_range_bounds(range)
                .expect("Buffer splat range is invalid");

            if start == end {
                return;
            }

            let borrow = &val;
            let offset = (start * size_of::<T>()) as isize;
            let size = ((end - start) * size_of::<T>()) as isize;
            gl::ClearNamedBufferSubData(
                self.buffer,
                gl::R8,
                offset,
                size,
                gl::RED,
                gl::UNSIGNED_BYTE,
                borrow as *const T as *const c_void,
            );
        }
    }

    // Extend the current buffer using data from a new slice
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        assert!(
            self.mode().write_permission(),
            "Cannot write to buffer, missing permission"
        );
        assert!(
            self.mode().modify_length_permission(),
            "Cannot extend buffer, missing permission"
        );

        unsafe {
            let ptr = if !slice.is_empty() {
                slice.as_ptr() as *const c_void
            } else {
                return;
            };
            let slice_byte_size = (slice.len() * size_of::<T>()) as isize;

            if self.length == 0 && self.capacity == 0 {
                // Allocate the buffer for the first time
                gl::NamedBufferData(self.buffer, slice_byte_size, ptr, gl::DYNAMIC_DRAW);
                self.length = slice.len();
                self.capacity = slice.len();
            } else if slice.len() + self.length > self.capacity {
                // Reallocate the buffer
                assert!(
                    self.mode().reallocate_permission(),
                    "Cannot reallocate buffer, missing reallocation permission"
                );

                // Some allocation values we need
                let new_capacity = (self.capacity + slice.len()) * 2;
                let new_length = self.length + slice.len();
                let new_capacity_byte_size = (new_capacity * size_of::<T>()) as isize;
                let old_capacity_byte_size = (self.capacity * size_of::<T>()) as isize;

                // Create temporary buffer that will store our old data
                let mut temp = 0;
                gl::CreateBuffers(1, &mut temp);
                gl::NamedBufferStorage(temp, old_capacity_byte_size, null(), 0);

                // Copy our current data into the temporary buffer
                gl::CopyNamedBufferSubData(self.buffer, temp, 0, 0, old_capacity_byte_size);

                // Reallocate the "self" buffer
                gl::NamedBufferData(
                    self.buffer,
                    new_capacity_byte_size,
                    null(),
                    gl::DYNAMIC_DRAW,
                );

                // Copy the data back from the temporary buffer
                gl::CopyNamedBufferSubData(temp, self.buffer, 0, 0, old_capacity_byte_size);
                gl::NamedBufferSubData(self.buffer, old_capacity_byte_size, slice_byte_size, ptr);

                // Delete the temporary buffer
                gl::DeleteBuffers(1, &temp);
                self.length = new_length;
                self.capacity = new_capacity;
            } else {
                // Update range sub-data
                let size = (slice.len() * size_of::<T>()) as isize;
                let offset = (self.length * size_of::<T>()) as isize;
                gl::NamedBufferSubData(self.buffer, offset, size, ptr);
                self.length += slice.len();
            }
        }
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn write_range(&mut self, slice: &[T], range: impl RangeBounds<usize>) {
        assert!(
            self.mode.write_permission(),
            "Cannot write to buffer, missing permission"
        );
        let (start, end) = self
            .convert_range_bounds(range)
            .expect("Buffer write range is invalid");
        assert_eq!(
            end - start,
            slice.len(),
            "Buffer write range is not equal to slice length"
        );

        let ptr = if !slice.is_empty() {
            slice.as_ptr() as *const c_void
        } else {
            return;
        };

        let offset = (start * size_of::<T>()) as isize;
        let size = ((end - start) * size_of::<T>()) as isize;

        unsafe {
            gl::NamedBufferSubData(self.buffer, offset, size, ptr);
        }
    }

    // Read a region of the buffer into a mutable slice
    pub fn read_range(&self, slice: &mut [T], range: impl RangeBounds<usize>) {
        let (start, end) = self
            .convert_range_bounds(range)
            .expect("Buffer read range is invalid");
        assert_eq!(
            end - start,
            slice.len(),
            "Buffer read range is not equal to slice length"
        );

        let offset = (start * size_of::<T>()) as isize;
        let size = ((end - start) * size_of::<T>()) as isize;

        unsafe {
            gl::GetNamedBufferSubData(self.buffer, offset, size, slice.as_mut_ptr() as *mut c_void);
        }
    }

    // Read a region of the buffer into a new vector
    pub fn read_range_as_vec(&self, range: impl RangeBounds<usize> + Copy) -> Vec<T> {
        let (start, end) = self
            .convert_range_bounds(range)
            .expect("Buffer read range is invalid");

        let mut vec = ManuallyDrop::new(vec![MaybeUninit::<T>::zeroed(); end - start]);
        self.read_range(
            unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut T, end - start) },
            range,
        );
        unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut T, end - start, end - start) }
    }

    // Read the whole buffer into a new vector
    pub fn read_to_vec(&self) -> Vec<T> {
        self.read_range_as_vec(..)
    }

    // Clear the buffer contents, resetting the buffer's length down to zero
    pub fn clear(&mut self) {
        assert!(
            self.mode().modify_length_permission(),
            "Cannot clear buffer, missing permission"
        );
        self.length = 0;
    }

    // Get an untyped buffer reference of the current buffer
    pub fn untyped_format(&self) -> UntypedBufferFormat {
        UntypedBufferFormat {
            target: TARGET,
            buffer: &self.buffer,
            length: &self.length,
            capacity: &self.capacity,
            mode: &self.mode,
            _type: TypeId::of::<T>(),
            stride: size_of::<T>(),
        }
    }

    // Get the buffer's stride (length of each element)
    pub fn stride(&self) -> usize {
        size_of::<T>()
    }

    // Cast the buffer to a buffer of another target / type
    // The type U and T must have the same exact size and alignment
    pub unsafe fn transmute<U: Shared, const OTHER: u32>(self) -> Buffer<U, OTHER> {
        assert_eq!(
            Layout::new::<T>(),
            Layout::new::<U>(),
            "Layout type mismatch, cannot cast buffer"
        );
        Buffer::<U, OTHER> {
            buffer: self.buffer,
            length: self.length,
            capacity: self.capacity,
            mode: self.mode,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        }
    }

    // Copy the data from another buffer's range into this buffer's range
    pub fn copy_range_from<const OTHER: u32>(&mut self, range: impl RangeBounds<usize>, other: &Buffer<T, OTHER>, offset: usize) {
        let (start, end) = other.convert_range_bounds(range).unwrap();
        assert!((end - start) + offset < self.length, "Cannot copy range from buffer, range is too big or offset is too large");

        unsafe {
            let size = ((end - start) * size_of::<T>()) as isize;
            let write_offset = (offset * size_of::<T>()) as isize;
            let read_offset = (start * size_of::<T>()) as isize;
            gl::CopyNamedBufferSubData(other.buffer, self.buffer, read_offset, write_offset, size);
        }
    }

    // Copy the data from another buffer into this buffer
    pub fn copy_from<const OTHER: u32>(&mut self, other: &Buffer<T, OTHER>) {
        assert_eq!(
            self.len(),
            other.len(),
            "Cannot copy from buffer, length mismatch"
        );
        unsafe {
            let size = (self.length * size_of::<T>()) as isize;
            gl::CopyNamedBufferSubData(other.buffer, self.buffer, 0, 0, size);
        }
    }

    // Copy the data from another buffer into this buffer, but transmute the other buffer as well
    pub unsafe fn copy_from_transmute<U: Shared, const OTHER: u32>(
        &mut self,
        other: &Buffer<U, OTHER>,
    ) {
        assert_eq!(
            self.len() * size_of::<T>(),
            other.len() * size_of::<U>(),
            "Cannot copy from buffer, byte size mismatch"
        );

        let size = (self.length * size_of::<T>()) as isize;
        gl::CopyNamedBufferSubData(other.buffer, self.buffer, 0, 0, size);
    }

    // Fills the whole buffer with a constant value
    pub fn splat(&mut self, val: T) {
        self.splat_range(val, ..)
    }

    // Overwrite the whole buffer using a slice
    pub fn write(&mut self, slice: &[T]) {
        self.write_range(slice, ..)
    }

    // Read the whole buffer into a mutable slice
    pub fn read(&self, slice: &mut [T]) {
        self.read_range(slice, ..)
    }

    // Create a new ranged buffer reader that can read from the buffer
    pub fn as_view_ranged(&self, range: impl RangeBounds<usize>) -> Option<BufferView<T, TARGET>> {
        if !self.mode.read_permission() {
            return None;
        }

        let already_mapped = unsafe {
            let mut mapped = 0i32;
            gl::GetNamedBufferParameteriv(self.buffer, gl::BUFFER_MAPPED, &mut mapped as *mut i32);
            mapped == 1i32
        };

        let (start, end) = self.convert_range_bounds(range)?;
        let should_map = self.mode.map_read_permission() && self.mode.map_persistent_permission();
        if should_map && !already_mapped {
            let offset = (start * size_of::<T>()) as isize;
            let size = ((end - start) * size_of::<T>()) as isize;

            let ptr = unsafe {
                gl::MapNamedBufferRange(
                    self.buffer,
                    offset,
                    size,
                    gl::MAP_READ_BIT | gl::MAP_PERSISTENT_BIT,
                ) as *const T
            };

            Some(BufferView::PersistentlyMapped {
                buf: self,
                range: (start, end),
                ptr,
            })
        } else {
            let vec = self.read_to_vec();
            Some(BufferView::Copied {
                buf: self,
                vec,
                range: (start, end),
            })
        }
    }

    // Create a new ranged buffer writer that can read/write from/to the buffer
    pub fn as_view_ranged_mut(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> Option<BufferViewMut<T, TARGET>> {
        if !self.mode.read_permission() || !self.mode.write_permission() {
            return None;
        }

        let (start, end) = self.convert_range_bounds(range)?;

        let should_map = self.mode.map_read_permission() && self.mode.map_write_permission();

        if should_map {
            let offset = (start * size_of::<T>()) as isize;
            let size = ((end - start) * size_of::<T>()) as isize;

            let ptr = unsafe {
                gl::MapNamedBufferRange(
                    self.buffer,
                    offset,
                    size,
                    gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
                ) as *mut T
            };

            Some(BufferViewMut::Mapped {
                buf: self,
                range: (start, end),
                ptr,
            })
        } else {
            let vec = self.read_to_vec();
            Some(BufferViewMut::Copied {
                buf: self,
                vec,
                range: (start, end),
            })
        }
    }

    // Create a buffer reader that uses the whole buffer
    pub fn as_view(&self) -> Option<BufferView<T, TARGET>> {
        self.as_view_ranged(..)
    }

    // Create a buffer writer that uses the whole buffer
    pub fn as_mut_view(&mut self) -> Option<BufferViewMut<T, TARGET>> {
        self.as_view_ranged_mut(..)
    }

    // Check if we can convert this to a persistent buffer
    pub fn can_convert_to_persistent(&self) -> bool {
        self.mode.map_read_permission()
            && self.mode.map_write_permission()
            && self.mode.map_persistent_permission()
            && !self.is_empty()
    }

    // Map the whole buffer persistently for reading/writing
    pub fn into_persistent(self) -> Option<(Persistent<T, TARGET>, PersistentAccessor<T, TARGET>)> {
        if !self.can_convert_to_persistent() {
            return None;
        }

        let size = (self.len() * size_of::<T>()) as isize;

        let ptr = unsafe {
            gl::MapNamedBufferRange(
                self.buffer,
                0,
                size,
                gl::MAP_READ_BIT
                    | gl::MAP_WRITE_BIT
                    | gl::MAP_PERSISTENT_BIT
                    | gl::MAP_FLUSH_EXPLICIT_BIT,
            ) as *mut T
        };

        let used = Arc::new(AtomicBool::new(false));

        let accessor = PersistentAccessor {
            len: self.len(),
            used: used.clone(),
            ptr,
        };

        let owner = Persistent {
            buf: Some(self),
            used,
        };

        Some((owner, accessor))
    }
}

impl<T: Shared, const TARGET: u32> ToGlName for Buffer<T, TARGET> {
    fn name(&self) -> u32 {
        self.buffer
    }
}

impl<T: Shared, const TARGET: u32> ToGlTarget for Buffer<T, TARGET> {
    fn target() -> u32 {
        TARGET
    }
}

impl<T: Shared, const TARGET: u32> Drop for Buffer<T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer);
        }
    }
}
