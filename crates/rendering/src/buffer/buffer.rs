use crate::context::{Context, ToGlName, ToGlType, Bind, Active};
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU32,
    ops::Range,
    ptr::null,
};

use super::BufferAccess;

// Objects that can be sent to the CPU
// TODO: Rename
pub trait GPUSendable: Copy + Sized + Sync + Send {}
impl<T: Copy + Sized + Sync + Send> GPUSendable for T {}

// Common OpenGL buffer types
pub type ArrayBuffer<T> = Buffer<T, { gl::ARRAY_BUFFER }>;
pub type ElementBuffer = Buffer<u32, { gl::ELEMENT_ARRAY_BUFFER }>;
pub type AtomicBuffer = Buffer<u32, { gl::ATOMIC_COUNTER_BUFFER }>;
pub type ComputeStorage<T> = Buffer<T, { gl::SHADER_STORAGE_BUFFER }>;
pub type UniformBuffer<T> = Buffer<T, { gl::UNIFORM_BUFFER }>;

// Buffers can be split into two types; dynamic buffers, and immutable buffers.
// TODO: Rename
enum Specification {
    // Immutable buffers can be allocated only once
    Immutable(u32),

    // Dynamic buffers can be allocated multiple times
    Dynamic(u32),
}

impl Specification {
    // Convert a buffer access to a specification
    pub fn new(access: BufferAccess) -> Self {
        if access.contains(BufferAccess::DYNAMIC) {
            // Mutable; can reallocate; meaning that the u32 represents buffer usage hints
            // TODO: Check if OpenGL actually cares about the hints lol
            let hints = if access.contains(BufferAccess::READ) {
                gl::DYNAMIC_READ
            } else if access.contains(BufferAccess::WRITE) {
                gl::DYNAMIC_DRAW
            } else {
                gl::DYNAMIC_COPY
            };
            Self::Dynamic(hints)
        } else {
            // Immutable; cannot reallocate; meaning that the u32 represents immutable storage flags
            let write = u32::from(access.contains(BufferAccess::WRITE)) * gl::MAP_WRITE_BIT;
            let read = u32::from(access.contains(BufferAccess::READ)) * gl::MAP_READ_BIT;
            let flags = write | read;
            Self::Immutable(flags)
        }
    }
}

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: GPUSendable, const TARGET: u32> {
    // OpenGL buffer name
    buffer: NonZeroU32,

    // Rust side values
    len: usize,
    capacity: usize,

    // This tells us if this is an immutable or resizable buffer
    spec: Specification,

    // Unsend + unsync
    _phantom: PhantomData<*const T>,
}

impl<T: GPUSendable, const TARGET: u32> Buffer<T, TARGET> {
    // Create a new buffer from it's raw parts
    pub unsafe fn from_raw_parts(_ctx: &Context, access: BufferAccess, length: usize, capacity: usize, ptr: *const T) -> Self {
        // Create the new OpenGL buffer
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);

        // Convert length and capacity to byte length and byte capacity
        //let byte_length = isize::try_from(length * size_of::<T>()).unwrap();
        let byte_capacity = isize::try_from(capacity * size_of::<T>()).unwrap();

        // Validate the pointer
        let ptr = if capacity == 0 { null() } else { ptr as *const c_void };

        // Convert the buffer access to the valid usage/flags
        let spec = Specification::new(access);

        match spec {
            Specification::Immutable(flags) => {
                // Upload immutable data to the GPU. Immutable buffers cannot be reallocated
                gl::NamedBufferStorage(buffer, byte_capacity, ptr as _, flags);
            }
            Specification::Dynamic(usage) => {
                // Upload mutable data to the GPU. Mutable buffers can be resized and reallocated
                gl::NamedBufferData(buffer, byte_capacity, ptr as _, usage);
            }
        }

        // Create the buffer struct
        Self {
            buffer: NonZeroU32::new(buffer).unwrap(),
            len: length,
            capacity,
            spec: Specification::Dynamic(0),
            _phantom: Default::default(),
        }
    }

    // Create a buffer with a specific starting capacity
    pub fn with_capacity(ctx: &mut Context, access: BufferAccess, capacity: usize) -> Self {
        unsafe { Self::from_raw_parts(ctx, access, 0, capacity, null()) }
    }

    // Create an empty buffer
    pub fn new(ctx: &mut Context, access: BufferAccess) -> Self {
        unsafe { Self::from_raw_parts(ctx, access, 0, 0, null()) }
    }

    // Create a buffer from a vector, and make sure the vector is not dropped before we send it's data to the GPU
    pub fn from_vec(ctx: &mut Context, access: BufferAccess, vec: Vec<T>) -> Self {
        unsafe {
            let mut manual = ManuallyDrop::new(vec);
            let me = Self::from_raw_parts(ctx, access, manual.len(), manual.capacity(), manual.as_ptr());
            ManuallyDrop::drop(&mut manual);
            me
        }
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.len
    }
    
    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Given an element index range, return the offset/length tuple
    fn validate(&self, range: Range<usize>) -> Option<(usize, usize)> {
        // Check if the range encapsulates the full range of the buffer
        let valid = range.end >= self.len || range.start >= self.len;
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
            buf: self.buffer,
            length,
            _phantom: Default::default(),
        })
    }

    // Using a range of elements, we shall make a mutable mapped buffer
    pub fn try_map_range_mut(&mut self, _ctx: &mut Context, range: Range<usize>) -> Option<MutMapped<T>> {
        self.validate(range).map(|(offset, length)| MutMapped {
            ptr: self.map_raw_unchecked(offset, length),
            buf: self.buffer,
            length,
            _phantom: Default::default(),
        })
    }

    // Overwrite the WHOLE buffer with some new values given from a vector
    pub fn overwrite(&mut self, ctx: &mut Context, new: Vec<T>) {
        let new_capacity = new.capacity();
        let new_length = new.len();

        match self.spec {
            Specification::Immutable(_) => {
                // Oopsie woopsie, uwu we made a fucky wucky, a little fucko-boingo
                assert!(new_capacity < self.capacity, "Cannot reallocate immutable buffer");

                // We should just update subdata through the mapper
                let mapped = self.try_map_range_mut(ctx, 0..new.len()).unwrap();
                let output = mapped.as_slice_mut();

                // Overwrite "putput" using elements from "new"
                unsafe {
                    // I use unsafe cause idk how to do it safely lul
                    let mut manual = ManuallyDrop::new(new);
                    std::ptr::copy(manual.as_ptr(), output.as_mut_ptr(), new_length);
                    ManuallyDrop::drop(&mut manual);
                }
            }
            Specification::Dynamic(usage) => {
                // Simply reallocate the buffer, since we know it is dynamic
                unsafe {
                    let mut manual = ManuallyDrop::new(new);
                    let byte_capacity = isize::try_from(new_capacity * size_of::<T>()).unwrap();
                    gl::NamedBufferData(self.buffer.get(), byte_capacity, manual.as_ptr() as _, usage);
                    ManuallyDrop::drop(&mut manual);
                }
            }
        }

        self.capacity = new_capacity;
        self.len = new_length;
    }

    // Add values to the end of the buffer, and reallocate it if needed
    pub fn extend_by_slice(&mut self, ctx: &mut Context, slice: &[T]) {
        // Calculate the new capacity only if the current cap + new len overflow
        let new_capacity = if self.capacity + slice.len() > self.len {
            (self.capacity + slice.len()).next_power_of_two()
        } else {
            self.capacity
        };

        // Calculate the new length as well
        let new_length = self.len + slice.len();

        match self.spec {
            Specification::Immutable(_) => {
                // Oopsie woopsie, uwu we made a fucky wucky, a little fucko-boingo
                assert!(new_capacity < self.capacity, "Cannot reallocate immutable buffer");

                // Update the subdata, starting from the end of the current buffer
                let mapped = self.try_map_range_mut(ctx, self.len..new_length).unwrap();
                let output = mapped.as_slice_mut();

                // Overwrite "output" using elements from "slice"
                unsafe {
                    std::ptr::copy(slice.as_ptr(), output.as_mut_ptr(), slice.len());
                }
            }
            Specification::Dynamic(usage) => {
                // Reallocate the whole dynamic buffer
                unsafe {
                    let byte_capacity = isize::try_from(new_capacity * size_of::<T>()).unwrap();
                    gl::NamedBufferData(self.buffer.get(), byte_capacity, slice.as_ptr() as _, usage);
                }
            }
        }

        self.capacity = new_capacity;
        self.len = new_length;
    }

    // Pop an element from the back of the buffer
    pub fn pop(&mut self, _ctx: &mut Context) -> Option<()> {
        self.len -= self.len.checked_sub(1)?;
        Some(())
    }
}

impl<T: GPUSendable, const TARGET: u32> ToGlName for Buffer<T, TARGET> {
    fn name(&self) -> NonZeroU32 {
        self.buffer
    }
}

impl<T: GPUSendable, const TARGET: u32> ToGlType for Buffer<T, TARGET> {
    fn target(&self) -> u32 {
        TARGET
    }
}

impl<T: GPUSendable, const TARGET: u32> Bind for Buffer<T, TARGET> {
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        unsafe {
            let target = self.target();
            gl::BindBuffer(target, self.buffer.get());
            function(Active(self));
        }
    }
}

impl<T: GPUSendable, const TARGET: u32> Drop for Buffer<T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer.get());
        }
    }
}

// An immutable mapped buffer that we can use to read data from the OpenGL buffer
pub struct RefMapped<'a, T: GPUSendable> {
    _phantom: PhantomData<&'a [T]>,
    buf: NonZeroU32,
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
            let res = gl::UnmapNamedBuffer(self.buf.get());
            assert!(res == gl::TRUE);
        }
    }
}

// A mutable mapped buffer that we can use to write/read to/from the OpenGL buffer
pub struct MutMapped<'a, T: GPUSendable> {
    _phantom: PhantomData<&'a [T]>,
    buf: NonZeroU32,
    ptr: *mut T,
    length: usize,
}

impl<'a, T: GPUSendable> Drop for MutMapped<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let res = gl::UnmapNamedBuffer(self.buf.get());
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
