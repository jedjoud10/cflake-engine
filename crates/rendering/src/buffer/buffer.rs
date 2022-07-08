use crate::object::{ToGlName, ToGlTarget};
use crate::{context::Context, object::Shared};
use std::mem::MaybeUninit;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};
use super::BufferError;

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Dynamic buffers can be modified, and they can push only a fixed amount of elements into their allocated space
    // They allocate only once, but the number of elements can vary, until they hit the capacity
    Dynamic,

    // Just like dynamic buffers, but resizable
    // By default, resizable buffers can be mapped
    Resizable,
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
    len: usize,
    capacity: usize,
    mode: BufferMode,

    _phantom: PhantomData<*const MaybeUninit<T>>,
    _phantom2: PhantomData<T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a new buffer from it's raw parts, like a pointer and some capacity and length
    // It should be valid to read from ptr for capacity number of elements (MaybeUninit<T>)
    // Capacity is the finaly allocated number of elements in the back store
    // Capacity should be greater or equal to length
    unsafe fn from_raw_parts(
        _ctx: &mut Context,
        mode: BufferMode,
        capacity: usize,
        length: usize,
        ptr: *const T,
    ) -> Result<Self, BufferError> {
        // Create the new OpenGL buffer
        let mut buffer = 0;
        gl::CreateBuffers(1, &mut buffer);

        // Convert size to byte size
        let bytes = isize::try_from(capacity * size_of::<T>()).unwrap();

        // We must some pre-existing data if we are a dynamic or static buffer
        if bytes == 0 {
            match mode {
                BufferMode::Static | BufferMode::StaticMapRead => Err(BufferError::EmptyStaticInit),
                BufferMode::DynamicMapReadWrite | BufferMode::Dynamic => Err(BufferError::EmptyDynamicInit),
                _ => Ok(())
            }?;
        }

        // Validate the pointer
        let ptr = if bytes == 0 {
            null()
        } else {
            ptr as *const c_void
        };

        // Initialize the buffer correctly
        match mode {
            BufferMode::Static => gl::NamedBufferStorage(buffer, bytes, ptr, 0),
            BufferMode::StaticMapRead => gl::NamedBufferStorage(buffer, bytes, ptr, gl::MAP_READ_BIT),
            BufferMode::Dynamic => gl::NamedBufferStorage(
                buffer,
                bytes,
                ptr,
                gl::DYNAMIC_STORAGE_BIT,
            ),
            BufferMode::DynamicMapReadWrite => gl::NamedBufferStorage(
                buffer,
                bytes,
                ptr,
                gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
            ),
            BufferMode::Resizable => gl::NamedBufferData(buffer, bytes, ptr, gl::DYNAMIC_DRAW),
        }

        // Create the buffer struct
        Ok(Self {
            buffer,
            len: length,
            capacity,
            mode,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        })
    }

    // Check if the buffer can be mutated by any means 
    pub fn can_write(&self) -> bool {
        match self.mode {
            BufferMode::Static => false,
            BufferMode::StaticMapRead => false,
            BufferMode::Dynamic => true,
            BufferMode::DynamicMapReadWrite => true,
            BufferMode::Resizable => true,

            AccesMode::None
        }
    }
    
    // Check if the buffer can be resized
    pub fn can_resize(&self) -> bool {
        match self.mode {
            BufferMode::Static => todo!(),
            BufferMode::StaticMapRead => todo!(),
            BufferMode::Dynamic => todo!(),
            BufferMode::DynamicMapReadWrite => todo!(),
            BufferMode::Resizable => todo!(),
        }
    }

    // Create an empty resizable buffer
    pub fn empty(_ctx: &mut Context) -> Self {
        unsafe { Self::from_raw_parts(_ctx, BufferMode::Resizable, 0, 0, null()) }.unwrap()
    }

    // Create a buffer using a buffer mode and a slice containing some data
    pub fn new(_ctx: &mut Context, mode: BufferMode, data: &[T]) -> Result<Self, BufferError> {
        unsafe { Self::from_raw_parts(_ctx, mode, data.len(), data.len(), data.as_ptr()) }
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Get the buffer mode that we used to initialize this buffer
    pub fn mode(&self) -> BufferMode {
        self.mode
    }

    // Try to clear the whole buffer to the given value
    pub fn clear(&mut self, val: T) -> Result<(), BufferError> {
        if let BufferMode::Static = self.mode {
            return Err(BufferError::WriteStatic);
        }

        unsafe {
            let bytes = isize::try_from(self.len() * size_of::<T>()).unwrap();
            if bytes != 0 {
                let borrow = &val as *const T;
                gl::ClearNamedBufferSubData(
                    self.buffer,
                    gl::R8,
                    0,
                    bytes,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    borrow as _,
                );
            }
        }

        Ok(())
    }

    // Extend the buffer with slice containing some new elements
    pub fn extend_from_slice(&mut self, slice: &[T]) -> Result<(), BufferError> {
        match self.mode {
            BufferMode::Static => todo!(),
            BufferMode::StaticMapRead => todo!(),
            BufferMode::Dynamic => todo!(),
            BufferMode::DynamicMapReadWrite => todo!(),
            BufferMode::Resizable => todo!(),
        }

        unsafe {
            let realloc = slice.len() + self.capacity() > self.capacity();

            if realloc {
                match self.mode {
                    BufferMode::Static => todo!(),
                    BufferMode::StaticMapRead => todo!(),
                    BufferMode::Dynamic => todo!(),
                    BufferMode::DynamicMapReadWrite => todo!(),
                    BufferMode::Resizable => todo!(),
                }
            }

            if realloc {
                //gl::NamedBufferData(self.buffer, bytes, data.as_ptr() as _, gl::DYNAMIC_DRAW);
            } else {

            }
        }
        Ok(())
    }

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