use crate::context::CommandStream;
use crate::{context::Context, object::Shared};
use crate::object::{self, ToGlName, ToGlType, Bind, Active};
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    num::NonZeroU32,
    ops::Range,
    ptr::null,
};

// Some settings that tell us how exactly we should create the buffer
#[derive(Clone, Copy)]
pub enum BufferMode {
    // glBufferStorage, immutable / unresizable
    // Static buffers can only be set once through their initialization
    Static,

    // glBufferStorage + GL_DYNAMIC_STORAGE_BIT + GL_CLIENT_STORAGE_BIT
    // Dynamic buffers can be modified, though they have a specific number of elements that must be constant
    Dynamic,

    // glBufferData + GL_DYNAMIC_DRAW 
    // Just like dynamic buffers, but resizable
    Resizable
}

// Simply used for organization
enum BufferType {
    // Immutable buffers allocated through glBufferStorage
    Immutable(u32),

    // Normal buffers allocated through glBufferData
    Default(u32)
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

    // Unsend + unsync
    _phantom: PhantomData<*const T>,
}

impl<T: Shared, const TARGET: u32> Buffer<T, TARGET> {
    // Create a new buffer from it's raw parts, like a pointer and some capacity and length
    unsafe fn from_raw_parts(ctx: &mut Context, _type: BufferType, capacity: usize, length: usize, ptr: *const T) -> Self {
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
            match _type {
                BufferType::Immutable(flags) => gl::NamedBufferStorage(buffer, bytes, ptr, flags),
                BufferType::Default(hints) => gl::NamedBufferData(buffer, bytes, ptr, hints),
            } 

            // Create the buffer struct
            Self {
                buffer: NonZeroU32::new(buffer).unwrap(),
                len: length,
                capacity,
                _phantom: Default::default(),
            }
        });
        
        // Await, basically
        cmd.wait(ctx)
    }

    // Lil wrapper around from_raw_parts
    unsafe fn setup(_ctx: &mut Context, _type: BufferType, data: &[T]) -> Self {
        Self::from_raw_parts(_ctx, _type, data.len(), data.len(), data.as_ptr())
    }

    // Create a static/immutable buffer
    unsafe fn immutable(_ctx: &mut Context, data: &[T]) -> Self {
        Self::setup(_ctx, BufferType::Immutable(0), data)
    }

    // Create a dynamic buffer
    unsafe fn dynamic(_ctx: &mut Context, data: &[T]) -> Self {
        Self::setup(_ctx, BufferType::Immutable(gl::DYNAMIC_STORAGE_BIT | gl::CLIENT_STORAGE_BIT), data)
    }

    // Create a resizable buffer
    unsafe fn resizable(_ctx: &mut Context, data: &[T]) -> Self {
        Self::setup(_ctx, BufferType::Default(gl::DYNAMIC_DRAW), data)
    }

    // Create a buffer using a buffer mode and a slice containing some data
    pub fn new(_ctx: &mut Context, mode: BufferMode, data: &[T]) -> Self {
        let func = match mode {
            BufferMode::Static => Self::immutable,
            BufferMode::Dynamic => Self::dynamic,
            BufferMode::Resizable => Self::resizable,
        };

        unsafe { func(_ctx, data) }
    }

    // Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.len
    }
    
    // Get the current capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /*
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
    */
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
            function(Active {
                inner: self,
                context: _ctx,
            });
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
