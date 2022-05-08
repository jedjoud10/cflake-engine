use crate::Context;
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ops::Range,
    ptr::{null, NonNull},
};

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
        let ptr = if capacity == 0 { null() } else { ptr as *const c_void };

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

    // Create a buffer with a specific starting capacity
    pub fn with_capacity(_ctx: &mut Context, immutable: bool, capacity: usize) -> Self {
        unsafe { Self::from_raw_parts(_ctx, immutable, 0, capacity, null()) }
    }

    // Create an empty buffer
    pub fn new(_ctx: &mut Context, immutable: bool) -> Self {
        unsafe { Self::from_raw_parts(_ctx, immutable, 0, 0, null()) }
    }

    // Create a buffer from a vector, and make sure the vector is not dropped before we send it's data to the GPU
    pub fn from_vec(_ctx: &mut Context, immutable: bool, vec: Vec<T>) -> Self {
        unsafe {
            let mut manual = ManuallyDrop::new(vec);
            let me = Self::from_raw_parts(_ctx, immutable, manual.len(), manual.capacity(), manual.as_ptr());
            ManuallyDrop::drop(&mut manual);
            me
        }
    }

    // Given an element index range, return the offset/length tuple
    fn validate(&self, range: Range<usize>) -> Option<(usize, usize)> {
        // Check if the range encapsulates the full range of the buffer
        let valid = range.end >= self.length || range.start >= self.length;
        (valid).then(|| {
            // Calculate offset and length
            let offset = range.start;
            let length = range.end - range.start;
            (offset, length)
        })
    }

    // Bind the buffer temporarily to a specific target, and unbind it when done
    pub fn bind(&mut self, _ctx: &mut Context, target: u32, f: impl FnOnce(&Self, u32)) {
        unsafe {
            gl::BindBuffer(target, self.buffer);
            f(self, self.buffer);
            gl::BindBuffer(target, 0);
        }
    }

    // Map the OpenGL buffer directly, without checking anything and without drop safety
    fn map_raw_unchecked(&self, offset: usize, length: usize) -> *mut T {
        unsafe {
            // Convert the element indices to byte indices
            let offset = isize::try_from(offset * size_of::<T>()).unwrap();
            let length = isize::try_from(length * size_of::<T>()).unwrap();
            let access = 0;
            gl::MapNamedBufferRange(self.buffer, offset, length, access) as _
        }
    }

    // Using a range of elements, we shall make a mapped buffer
    pub fn try_map_range(&self, range: Range<usize>) -> Option<RefMapped<T>> {
        self.validate(range).map(|(offset, length)| RefMapped {
            ptr: self.map_raw_unchecked(offset, length),
            buf: self,
            length,
        })
    }

    // Using a range of elements, we shall make a mutable mapped buffer
    pub fn try_map_range_mut(&mut self, range: Range<usize>) -> Option<MutMapped<T>> {
        self.validate(range).map(|(offset, length)| MutMapped {
            ptr: self.map_raw_unchecked(offset, length),
            buf: self,
            length,
        })
    }
}

// An immutable mapped buffer that we can use to read data from the OpenGL buffer
pub struct RefMapped<'a, T: GPUSendable> {
    buf: &'a Buffer<T>,
    ptr: *const T,
    length: usize,
}

impl<'a, T: GPUSendable> RefMapped<'a, T> {
    // Create an immutable slice from the mapped region
    pub fn as_slice(&self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }
}

impl<'a, T: GPUSendable> Drop for RefMapped<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let res = gl::UnmapNamedBuffer(self.buf.buffer);
            assert!(res == gl::TRUE);
        }
    }
}

// A mutable mapped buffer that we can use to write/read to/from the OpenGL buffer
pub struct MutMapped<'a, T: GPUSendable> {
    buf: &'a mut Buffer<T>,
    ptr: *mut T,
    length: usize,
}

impl<'a, T: GPUSendable> Drop for MutMapped<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let res = gl::UnmapNamedBuffer(self.buf.buffer);
            assert!(res == gl::TRUE);
        }
    }
}

impl<'a, T: GPUSendable> MutMapped<'a, T> {
    // Create an immutable slice from the mapped region
    pub fn as_slice(&self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }

    // Create a mutable slice from the mapped region
    pub fn as_lice_mut(&self) -> &'a mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.length) }
    }
}
