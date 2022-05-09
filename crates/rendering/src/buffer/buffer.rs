use crate::context::Context;
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU32,
    ops::Range,
    ptr::null,
};

use super::BufferTarget;

// Objects that can be sent to the CPU
// TODO: Rename
pub trait GPUSendable: Copy + Sized + Sync + Send {}
impl<T: Copy + Sized + Sync + Send> GPUSendable for T {}

// An abstraction layer over a valid OpenGL buffer
pub struct Buffer<Target: BufferTarget> {
    // OpenGL buffer data
    buffer: NonZeroU32,
    length: usize,
    capacity: usize,
    immutable: bool,

    _phantom: PhantomData<*const Target>,
}

impl<Target: BufferTarget> Buffer<T> {
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
            buffer: NonZeroU32::new(buffer).unwrap(),
            length,
            capacity,
            immutable,
            _phantom: Default::default(),
        }
    }

    // Create a buffer with a specific starting capacity
    pub fn with_capacity(ctx: &mut Context, immutable: bool, capacity: usize) -> Self {
        unsafe { Self::from_raw_parts(ctx, immutable, 0, capacity, null()) }
    }

    // Create an empty buffer
    pub fn new(ctx: &mut Context, immutable: bool) -> Self {
        unsafe { Self::from_raw_parts(ctx, immutable, 0, 0, null()) }
    }

    // Create a buffer from a vector, and make sure the vector is not dropped before we send it's data to the GPU
    pub fn from_vec(ctx: &mut Context, immutable: bool, vec: Vec<T>) -> Self {
        unsafe {
            let mut manual = ManuallyDrop::new(vec);
            let me = Self::from_raw_parts(ctx, immutable, manual.len(), manual.capacity(), manual.as_ptr());
            ManuallyDrop::drop(&mut manual);
            me
        }
    }

    // Bind the buffer temporarily to a specific target, and unbind it when done
    pub fn bind(&mut self, _ctx: &mut Context, target: NonZeroU32, f: impl FnOnce(&Self, u32)) {
        unsafe {
            gl::BindBuffer(target.get(), self.buffer.get());
            f(self, self.buffer.get());
            gl::BindBuffer(target.get(), 0);
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

    // Map the OpenGL buffer directly, without checking anything and without drop safety
    fn map_raw_unchecked(&self, offset: usize, length: usize) -> *mut T {
        unsafe {
            // Convert the element indices to byte indices
            let offset = isize::try_from(offset * size_of::<T>()).unwrap();
            let length = isize::try_from(length * size_of::<T>()).unwrap();
            let access = 0;
            gl::MapNamedBufferRange(self.buffer.get(), offset, length, access) as _
        }
    }

    // Using a range of elements, we shall make a mapped buffer
    pub fn try_map_range(&self, _ctx: &Context, range: Range<usize>) -> Option<RefMapped<T>> {
        self.validate(range).map(|(offset, length)| RefMapped {
            ptr: self.map_raw_unchecked(offset, length),
            buf: self,
            length,
        })
    }

    // Using a range of elements, we shall make a mutable mapped buffer
    pub fn try_map_range_mut(&mut self, _ctx: &mut Context, range: Range<usize>) -> Option<MutMapped<T>> {
        self.validate(range).map(|(offset, length)| MutMapped {
            ptr: self.map_raw_unchecked(offset, length),
            buf: self,
            length,
        })
    }

    // Overwrite the buffer with some new values
    pub fn overwrite(&mut self, ctx: &mut Context, new: Vec<T>) {
        // Keep these values cached
        let new_capacity = new.capacity();
        let new_length = new.len();

        // Check if we can fill the data without having to reallocate
        if new_capacity <= self.capacity {
            // We should just update subdata through the mapper
            let mapped = self.try_map_range_mut(ctx, 0..new.len()).unwrap();
            let slice = mapped.as_slice_mut();

            // Overwrite "slice" using elements from "new"
            unsafe {
                // I use unsafe cause idk how to do it safely lul
                let mut manual = ManuallyDrop::new(new);
                std::ptr::copy(manual.as_ptr(), slice.as_mut_ptr(), new_length);
                ManuallyDrop::drop(&mut manual);
            }
        } else {
            // We must reallocate
            if self.immutable {
                // Oopsie woopsie, uwu we made a fucky wucky, a little fucko-boingo
                panic!()
            } else {
                // Simply reallocate the buffer
                unsafe {
                    let mut manual = ManuallyDrop::new(new);
                    let byte_capacity = isize::try_from(new_capacity * size_of::<T>()).unwrap();
                    gl::NamedBufferData(self.buffer.get(), byte_capacity, manual.as_ptr() as _, 0);
                    ManuallyDrop::drop(&mut manual);
                }
            }
        }

        // Funny rust length and capacity moment
        self.capacity = new_capacity;
        self.length = new_length;
    }

    // Add values to the end of the buffer, and reallocate it if needed
}

// An immutable mapped buffer that we can use to read data from the OpenGL buffer
pub struct RefMapped<'a, T: GPUSendable> {
    buf: &'a Buffer<T>,
    ptr: *const T,
    length: usize,
}

impl<'a, T: GPUSendable> RefMapped<'a, T> {
    // Create an immutable slice from the mapped region
    pub fn as_slice(&'a self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }
}

impl<'a, T: GPUSendable> Drop for RefMapped<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let res = gl::UnmapNamedBuffer(self.buf.buffer.get());
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
            let res = gl::UnmapNamedBuffer(self.buf.buffer.get());
            assert!(res == gl::TRUE);
        }
    }
}

impl<'a, T: GPUSendable> MutMapped<'a, T> {
    // Create an immutable slice from the mapped region
    pub fn as_slice(&'a self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }

    // Create a mutable slice from the mapped region
    pub fn as_slice_mut(&'a self) -> &'a mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.length) }
    }
}
