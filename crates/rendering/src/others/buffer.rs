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
    unsafe fn from_raw_parts(ctx: &mut Context, mode: BufferMode, capacity: usize, length: usize, ptr: *const T) -> Option<Self> {
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
        let ptr = if bytes == 0 { null() } else { ptr as *const c_void };

        // Initialize the buffer correctly
        match mode {
            BufferMode::Static => gl::NamedBufferStorage(buffer, bytes, ptr, 0),
            BufferMode::Dynamic => gl::NamedBufferStorage(buffer, bytes, ptr, gl::DYNAMIC_STORAGE_BIT | gl::CLIENT_STORAGE_BIT),
            BufferMode::Resizable => gl::NamedBufferData(buffer, bytes, ptr, gl::DYNAMIC_DRAW),
        }

        // Todo: Use a fence instead of this
        gl::Finish();

        // Create the buffer struct
        Some(Self {
            buffer: NonZeroU32::new(buffer).unwrap(),
            len: length,
            capacity,
            mode,
            _phantom: Default::default(),
        })
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
    pub fn clear(&mut self, ctx: &mut Context, val: T) {
        // Cannot clear static buffers
        assert_ne!(self.mode, BufferMode::Static, "Cannot clear Static buffers");

        unsafe {
            let bytes = isize::try_from(self.len() * size_of::<T>()).unwrap();
            if bytes != 0 {
                let borrow = &val as *const T;
                gl::ClearNamedBufferSubData(self.buffer.get(), gl::R8, 0, bytes, gl::RED, gl::UNSIGNED_BYTE, borrow as _);
            }
        }
    }

    // Overwrite the whole buffer if possible
    pub fn update(&mut self, ctx: &mut Context, data: &[T]) {
        // Cannot update static buffers
        assert_ne!(self.mode, BufferMode::Static, "Cannot update Static buffers");

        // Make sure the lengths match up (in case of a dynamic buffer)
        assert!(self.mode == BufferMode::Resizable || data.len() == self.len());

        unsafe {
            self.len = data.len();
            self.capacity = data.len();
            let bytes = isize::try_from(data.len() * size_of::<T>()).unwrap();
            gl::NamedBufferSubData(self.buffer.get(), 0, bytes, data.as_ptr() as _);
        }
    }

    // Read back the whole buffer, and store it inside output
    pub fn read(&self, ctx: &Context, output: &mut [T]) {
        // Make sure the lengths always match up
        assert!(output.len() == self.len(), "Current length and output length do not match up.");

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
