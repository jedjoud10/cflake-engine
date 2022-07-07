use crate::object::{ToGlName, ToGlTarget};
use crate::{context::Context, object::Shared};
use std::mem::MaybeUninit;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // Static buffers can only be set once through their initialization
    Static,

    // Same as Static, but the buffer can be mapped and then read
    StaticMapRead,

    // Dynamic buffers can be modified, though they have a specific number of elements that must be constant
    Dynamic,

    // Same as dynamic, but the buffer can be mapped for reading/writing
    DynamicMapReadWrite,

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

    _phantom: PhantomData<*const T>,
    _phantom2: PhantomData<T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a new buffer from it's raw parts, like a pointer and some capacity and length
    unsafe fn from_raw_parts(
        _ctx: &mut Context,
        mode: BufferMode,
        capacity: usize,
        length: usize,
        ptr: *const T,
    ) -> Option<Self> {
        // Create the new OpenGL buffer
        let mut buffer = 0;
        gl::CreateBuffers(1, &mut buffer);

        // Convert size to byte size
        let bytes = isize::try_from(capacity * size_of::<T>()).unwrap();

        // We must some pre-existing data if we are a dynamic or static buffer
        if bytes == 0 && (mode == BufferMode::Dynamic || mode == BufferMode::Static) {
            return None;
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
        Some(Self {
            buffer,
            len: length,
            capacity,
            mode,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        })
    }

    // Create an empty resizable buffer
    pub fn empty(_ctx: &mut Context) -> Self {
        unsafe { Self::from_raw_parts(_ctx, BufferMode::Resizable, 0, 0, null()) }.unwrap()
    }

    // Create a buffer using a buffer mode and a slice containing some data
    pub fn new(_ctx: &mut Context, mode: BufferMode, data: &[T]) -> Option<Self> {
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
    pub fn clear(&mut self, val: T) {
        assert_ne!(self.mode, BufferMode::Static, "Cannot clear static buffers");

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
    }

    // Overwrite the whole buffer with new data
    pub fn write(&mut self, data: &[T]) {
        assert_ne!(
            self.mode,
            BufferMode::Static,
            "Cannot update Static buffers"
        );

        assert!(self.mode == BufferMode::Resizable || data.len() == self.len());

        unsafe {
            let bytes = isize::try_from(data.len() * size_of::<T>()).unwrap();

            // Reallocate if we need to, but if we don't need to just update a subregion of the buffer
            if data.len() > self.len() {
                gl::NamedBufferData(self.buffer, bytes, data.as_ptr() as _, gl::DYNAMIC_DRAW);
            } else {
                gl::NamedBufferSubData(self.buffer, 0, bytes, data.as_ptr() as _);
            }
        }

        // Update length and capacity states at the end
        self.len = data.len();
        self.capacity = data.len();
    }

    // Copy the data from another buffer into our buffer
    pub fn copy_from<U: Shared, const OTHER: u32>(&mut self, other: &Buffer<U, OTHER>) {
        assert!(
            self.len * size_of::<T>() == other.len(),
            "Current byte length and other buffer byte length do not match up, cannot copy"
        );

        unsafe {
            let bytes = isize::try_from(self.len() * size_of::<T>()).unwrap();
            gl::CopyNamedBufferSubData(other.name(), self.name(), 0, 0, bytes);
        }
    }

    // Copy the data from our buffer into another buffer
    pub fn copy_into<U: Shared, const OTHER: u32>(&self, other: &mut Buffer<U, OTHER>) {
        other.copy_from(self);
    }

    // Read back the whole buffer, and store it inside output
    pub fn read(&self, output: &mut [T]) {
        assert!(
            output.len() == self.len(),
            "Current length and output length do not match up, cannot read"
        );

        unsafe {
            let bytes = isize::try_from(output.len() * size_of::<T>()).unwrap();
            gl::GetNamedBufferSubData(self.buffer, 0, bytes, output.as_mut_ptr() as _);
        }
    }

    // Map the buffer, making it unusable for the duration of the mapping
    pub fn map(&mut self, access: MapAccess) -> Option<Mapped<T, TARGET>> {
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
            return None;
        }

        // Missing write permission
        if need_write && !write {

        }

        let ptr = unsafe {
            gl::MapNamedBuffer(self.buffer, ) as *mut MaybeUninit<T>
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