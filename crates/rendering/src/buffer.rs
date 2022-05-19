use crate::context::CommandStream;
use crate::object::{self, Active, Bind, ToGlName, ToGlType};
use crate::{context::Context, object::Shared};
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU32,
    ops::Range,
    ptr::null,
};

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferMode {
    // glBufferStorage, immutable / unresizable
    // Static buffers can only be set once through their initialization
    Static,

    // glBufferStorage + GL_DYNAMIC_STORAGE_BIT + GL_CLIENT_STORAGE_BIT
    // Dynamic buffers can be modified, though they have a specific number of elements that must be constant
    Dynamic,

    // glBufferData + GL_DYNAMIC_DRAW
    // Just like dynamic buffers, but resizable
    Resizable,
}

// Simply used for organization
enum BufferType {
    // Immutable buffers allocated through glBufferStorage
    Immutable(u32),

    // Normal buffers allocated through glBufferData
    Default(u32),
}

// Common OpenGL buffer types
pub type ArrayBuffer<T> = Buffer<T, { gl::ARRAY_BUFFER }>;
pub type ElementBuffer = Buffer<u32, { gl::ELEMENT_ARRAY_BUFFER }>;
pub type AtomicBuffer = Buffer<u32, { gl::ATOMIC_COUNTER_BUFFER }>;
pub type ComputeStorage<T> = Buffer<T, { gl::SHADER_STORAGE_BUFFER }>;
pub type UniformBuffer<T> = Buffer<T, { gl::UNIFORM_BUFFER }>;

// An abstraction layer over a valid OpenGL buffer
// This takes a valid OpenGL type and an element type, though the user won't be able make the buffer directly
// This also takes a constant that represents it's OpenGL target
pub struct Buffer<T: Shared, const TARGET: u32> {
    // OpenGL buffer name
    buffer: NonZeroU32,

    // I am slowly going insane
    len: usize,
    capacity: usize,
    mode: BufferMode,

    // Unsend + unsync
    _phantom: PhantomData<*const T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a new buffer from it's raw parts, like a pointer and some capacity and length
    unsafe fn from_raw_parts(ctx: &mut Context, mode: BufferMode, capacity: usize, length: usize, ptr: *const T) -> Self {
        // Create a command stream since we HAVE to make sure we still have the pointer in memory
        let cmd = CommandStream::new(ctx, |_| {
            // Create the new OpenGL buffer
            let mut buffer = 0;
            gl::GenBuffers(1, &mut buffer);

            // Convert size to byte size
            let bytes = isize::try_from(capacity * size_of::<T>()).unwrap();

            // Validate the pointer
            let ptr = if bytes == 0 { null() } else { ptr as *const c_void };

            // Initialize the buffer correctly
            match mode {
                BufferMode::Static => gl::NamedBufferStorage(buffer, bytes, ptr, 0),
                BufferMode::Dynamic => gl::NamedBufferStorage(buffer, bytes, ptr, gl::DYNAMIC_STORAGE_BIT | gl::CLIENT_STORAGE_BIT),
                BufferMode::Resizable => gl::NamedBufferData(buffer, bytes, ptr, gl::DYNAMIC_DRAW),
            }

            // Create the buffer struct
            Self {
                buffer: NonZeroU32::new(buffer).unwrap(),
                len: length,
                capacity,
                mode,
                _phantom: Default::default(),
            }
        });

        // Await, basically
        cmd.wait(ctx)
    }

    // Create a buffer using a buffer mode and a slice containing some data
    pub fn new(_ctx: &mut Context, mode: BufferMode, data: &[T]) -> Self {
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

    // Overwrite the whole buffer if possible
    pub fn update(&mut self, ctx: &mut Context, data: &[T]) {
        // Cannot update static buffers
        assert_ne!(self.mode, BufferMode::Static, "Cannot update buffers that were initialized using BufferMode::Static.");

        // Make sure the lengths match up (in case of a dynamic buffer)
        assert!(self.mode == BufferMode::Resizable || data.len() == self.len());

        // Generic update method
        unsafe {
            let bytes = isize::try_from(data.len() * size_of::<T>()).unwrap();
            gl::NamedBufferSubData(self.buffer.get(), 0, bytes, data.as_ptr() as _);
        }
    }

    // Read back the whole buffer, and store it inside output
    pub fn read(&self, ctx: &Context, output: &mut [T]) {
        // Make sure the lengths always match up
        assert!(output.len() == self.len(), "Current length and output length do not match up.");

        // Generic reading method
        unsafe {
            let bytes = isize::try_from(output.len() * size_of::<T>()).unwrap();
            gl::GetNamedBufferSubData(self.buffer.get(), 0, bytes, output.as_mut_ptr() as _);
        }
    }
}

impl<T: Shared, const TARGET: u32> ToGlName for Buffer<T, TARGET> {
    fn name(&self) -> NonZeroU32 {
        self.buffer
    }
}

impl<T: Shared, const TARGET: u32> ToGlType for Buffer<T, TARGET> {
    fn target(&self) -> u32 {
        TARGET
    }
}

impl<T: Shared, const TARGET: u32> Bind for Buffer<T, TARGET> {
    fn bind(&mut self, _ctx: &mut Context, function: impl FnOnce(Active<Self>)) {
        unsafe {
            let target = self.target();
            gl::BindBuffer(target, self.buffer.get());
            function(Active::new(self, _ctx));
        }
    }
}

impl<T: Shared, const TARGET: u32> Drop for Buffer<T, TARGET> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.buffer.get());
        }
    }
}
