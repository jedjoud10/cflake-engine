use crate::object::{ToGlName, ToGlTarget};
use crate::{context::Context, object::Shared};
use std::mem::MaybeUninit;
use std::ops::{Range, RangeBounds};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};
use super::BufferError;

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Static buffers are only created once, and they can never be modified ever again
    Static,

    // Dynamic buffers are like static buffers, but they allow the user to mutate each element
    Dynamic,

    // Partial buffer have a fixed capacity, but a dynamic length 
    Parital,

    // Resizable buffers can be resized to whatever length needed
    Resizable,
}


// Common OpenGL buffer types
pub type ArrayBuffer<T> = Buffer<T, { gl::ARRAY_BUFFER }>;
pub type ElementBuffer<T> = Buffer<T, { gl::ELEMENT_ARRAY_BUFFER }>;
pub type AtomicBuffer<T> = Buffer<T, { gl::ATOMIC_COUNTER_BUFFER }>;
pub type ComputeStorage<T> = Buffer<T, { gl::SHADER_STORAGE_BUFFER }>;
pub type UniformBuffer<T> = Buffer<T, { gl::UNIFORM_BUFFER }>;

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
    /*
    unsafe fn from_raw_parts(
        _ctx: &mut Context,
        mode: BufferMode,
        length: usize,
        capacity: usize,
        ptr: *const T,
    ) -> Result<Self, BufferError> {
        // Create the new OpenGL buffer
        let mut buffer = 0;
        gl::CreateBuffers(1, &mut buffer);

        // Capacity byte count and length byte count
        let byte_length = isize::try_from(length * size_of::<T>()).unwrap();
        let byte_capacity = isize::try_from(capacity * size_of::<T>()).unwrap();

        match mode {
            BufferMode::Static => byte_length > 0,
            BufferMode::Dynamic => byte_length > 0,
            BufferMode::Parital => byte_capacity > 0,
            BufferMode::Resizable => true,
        }

        // Validate the pointer
        let ptr = if byte_length == 0 {
            null()
        } else {
            ptr as *const c_void
        };

        // Initialize the buffer correctly
        match mode {
            BufferMode::Static => gl::NamedBufferStorage(buffer, bytes, ptr, 0),
            BufferMode::Dynamic => gl::NamedBufferStorage(
                buffer,
                bytes,
                ptr,
                gl::DYNAMIC_STORAGE_BIT,
            ),
            BufferMode::Parital => {
                gl::NamedBufferStorage(buffer, bytes)
            },
            BufferMode::Resizable => gl::NamedBufferData(buffer, bytes, ptr, gl::DYNAMIC_DRAW),
        }

        // Create the buffer struct
        Ok(Self {
            buffer,
            length,
            capacity,
            mode,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        })
    }
    */

    // Create a buffer using a slice of elements
    // The capacity of the buffer will automatically be set to the length of the slice
    pub fn try_from_slice(ctx: &mut Context, mode: BufferMode, data: &[T]) -> Result<Self, BufferError> {
        todo!()
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.length
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Clear the values specified by the range to a new value
    pub fn try_clear_range(&mut self, val: T, range: impl RangeBounds<usize>) -> Result<(), BufferError> {
        todo!()
    }

    // Clear the whole contents of the buffer to the specified value 
    pub fn try_clear(&mut self, val: T) -> Result<(), BufferError> {
        self.try_clear_range(val, ..)
    }

    // Extend the current buffer using data from a new slice
    pub fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), BufferError> {
        todo!()
    }

    // Push a single element into the buffer (slow!)
    pub fn try_push(&mut self, value: T) -> Result<(), BufferError> {
        todo!()
    }

    // Remove the last element from the buffer (slow!)
    pub fn try_pop(&mut self) -> Result<Option<T>, BufferError> {
        todo!()
    }

    // Overwrite a region of the buffer using a slice and a range
    pub fn try_write_range(&mut self, slice: &[T], range: impl RangeBounds<usize>) -> Result<(), BufferError> {
        todo!()
    }

    // Overwrite the whole buffer using a slice
    pub fn try_write(&mut self, slice: &[T]) -> Result<(), BufferError> {
        self.try_write_range(slice, ..)
    }

    // Read a region of the buffer into a mutable slice
    pub fn try_read_range(&mut self, slice: &mut [T], range: impl RangeBounds<usize>) -> Result<(), BufferError> {
        todo!()
    }

    // Read the whole buffer into a mutable slice
    pub fn try_read(&mut self, slice: &mut [T]) -> Result<(), BufferError> {
        self.try_read_range(slice, ..)
    }

    // Copy the buffer contents of Self into Other
    pub fn try_copy_into<U: Shared, const OTHER: u32>(&self, other: &mut Buffer<U, OTHER>) -> Result<(), BufferError> {
        todo!()
    }

    // Push a whole slice into the buffer (if it is resizable that is)

    /*
    
    */

    /*
    // Overwrite the whole buffer with new data. This will not extend the buffer automatically
    pub fn write_from_slice(&mut self, slice: &[T]) -> Result<(), BufferError> {
        if let BufferMode::Static = self.mode {
            return Err(BufferError::WriteStatic);
        }

        if slice.len() != self.len() {
            return Err(BufferError::WriteFromSliceInvalidLen(self.len(), slice.len()));
        }

        unsafe {
            let bytes = isize::try_from(slice.len() * size_of::<T>()).unwrap();
            gl::NamedBufferSubData(self.buffer, 0, bytes, slice.as_ptr() as _);
        }

        Ok(())
    }

    // Copy the data from another buffer into our buffer
    pub fn copy_from<U: Shared, const OTHER: u32>(&mut self, other: &Buffer<U, OTHER>) -> Result<(), BufferError> {
        if self.len * size_of::<T>() != other.len() * size_of::<U>() {
            return Err(BufferError::CopyFromInvalidLen(self.len() * size_of::<T>(), other.len() * size_of::<U>()));
        }

        unsafe {
            let bytes = isize::try_from(self.len() * size_of::<T>()).unwrap();
            gl::CopyNamedBufferSubData(other.name(), self.name(), 0, 0, bytes);
        }

        Ok(())
    }

    // Copy the data from our buffer into another buffer
    pub fn copy_into<U: Shared, const OTHER: u32>(&self, other: &mut Buffer<U, OTHER>) -> Result<(), BufferError> {
        other.copy_from(self).map_err(|err| BufferError::CopyToInvalidLen(self.len() * size_of::<T>(), other.len() * size_of::<U>()))
    }

    // Read back the whole buffer, and store it inside output
    pub fn read_slice(&self, output: &mut [T]) -> Result<(), BufferError> {
        if output.len() != self.len() {
            return Err(BufferError::ReadToSliceInvalidLen(self.len(), output.len()))
        }
        assert!(
            output.len() == self.len(),
            "Current length and output length do not match up, cannot read"
        );

        unsafe {
            let bytes = isize::try_from(output.len() * size_of::<T>()).unwrap();
            gl::GetNamedBufferSubData(self.buffer, 0, bytes, output.as_mut_ptr() as _);
        }

        Ok(())
    }
    */

    /*
    // Map the buffer, making it unusable for the duration of the mapping
    pub fn map(&mut self, access: MapAccess) -> Result<Mapped<T, TARGET>, BufferError> {
        // Read / write permission for the buffer
        let (read, write) = match self.mode {
            BufferMode::StaticMapRead => (true, false),
            BufferMode::DynamicMapReadWrite => (true, true),
            BufferMode::Resizable => (true, true),
            _ => (false, false)
        };

        // What the mapped buffer will be doing
        let (need_read, need_write) = match access {
            MapAccess::Read => (true, false),
            MapAccess::Write => (false, true),
            MapAccess::ReadWrite => (true, true),
        };

        // Missing read permission
        if need_read && !read {
            return Err(BufferError::MapPermissionsInvalidRead(access, self.mode));
        }

        // Missing write permission
        if need_write && !write {
            return Err(BufferError::MapPermissionsInvalidRead(access, self.mode));
        }

        let ptr = unsafe {
            gl::MapNamedBuffer(self.buffer, match access {
                MapAccess::Read => gl::MAP_READ_BIT,
                MapAccess::Write => gl::MAP_WRITE_BIT,
                MapAccess::ReadWrite => gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
            }) as *mut MaybeUninit<T>
        };
        
        /*
        Mapped {
            buffer: self,
            ptr,
        }
        */
        todo!()
    }
    */
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

// Some settings that specify what the user has access to whenever mapping a buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapAccess {
    // The user will be reading from the mapped buffer
    Read,
    
    // The user will be writing to the mapped buffer
    Write,
    
    // The user will be reading AND writing to the mapped buffer
    ReadWrite,
}
/*

// This is a mapped buffer. The memory of the mapped buffer might be stored within the RAM, so it would be faster to  
// Buffers that are mapped cannot be used in any way whilst they are mapped
pub struct Mapped<'a, T: Shared, const TARGET: u32> {
    buffer: &'a mut Buffer<T, TARGET>,
    ptr: *mut MaybeUninit<T>,
}

impl<'a, T: Shared, const TARGET: u32> Mapped<'a, T, TARGET> {
    // Read the mapped buffer as if it was an immutable slice
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr as *const T, self.buffer.len())
        }
    }

    // Read the mapped buffer as if it was a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            assert_ne!(self.buffer.mode, BufferMode::Static, "Cannot mutate static buffers");
            std::slice::from_raw_parts_mut(self.ptr as *mut T, self.buffer.len())
        }
    }    
}

impl<'a, T: Shared, const TARGET: u32> Drop for Mapped<'a, T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.buffer.buffer);
        }
    }
}
*/