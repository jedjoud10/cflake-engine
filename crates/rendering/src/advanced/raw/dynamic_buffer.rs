use crate::{
    basics::bufop::GLBufferOperations,
    object::OpenGLObjectNotInitialized,
    pipeline::Pipeline,
    utils::{
        AccessType::ServerToClient,
        UpdateFrequency::{Dynamic, Static},
        UsageType,
    },
};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ops::Range,
    ptr::null,
};

// A dynamic OpenGL buffer that automatically reallocates it's size when we add too many elements to it
#[derive(Getters, CopyGetters)]
pub struct DynamicRawBuffer<T> {
    // The OpenGL data for this buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get_copy = "pub")]
    _type: GLuint,

    // Other data
    #[getset(get_copy = "pub")]
    usage: UsageType,
    #[getset(get = "pub")]
    inner: Vec<T>,
    _phantom: PhantomData<*const ()>,
}

impl<T> Default for DynamicRawBuffer<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            _type: Default::default(),
            usage: UsageType::new(ServerToClient, Dynamic),
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

// Creation
impl<T> DynamicRawBuffer<T> {
    // Create the dynamic raw buffer
    pub fn new(_type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let oid = unsafe {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            oid
        };
        Self {
            buffer: oid,
            _type,
            usage,
            inner: Default::default(),
            _phantom: PhantomData::default(),
        }
    }
    // Create a new dynamic raw buffer with a specified length
    // TODO: FIX THEIS
    pub fn with_length(_type: u32, length: usize, usage: UsageType, _pipeline: &Pipeline) -> Self
    where
        T: Copy + Default,
    {
        let vec = vec![T::default(); length];
        let oid = unsafe {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            gl::BindBuffer(_type, oid);
            if length != 0 {
                gl::BufferData(_type, (size_of::<T>() * length) as isize, null(), usage.convert());
            }
            gl::BindBuffer(_type, 0);
            oid
        };
        Self {
            buffer: oid,
            _type,
            inner: vec,
            usage,
            _phantom: PhantomData::default(),
        }
    }
}

// Allocation
impl<T> DynamicRawBuffer<T> {
    // Reallocate the buffer
    fn reallocate(&self) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(
                self._type,
                (size_of::<T>() * self.inner.capacity()) as isize,
                self.inner.as_ptr() as *const c_void,
                self.usage.convert(),
            );
            gl::BindBuffer(self._type, 0);
        }
    }
    // Update the buffer data in the specified range (elements)
    fn update(&self, range: Range<usize>) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            // Borth range indices correspond to elements' indices, not their byte offset
            let start = range.start;
            let end = range.end;

            // Byte offset and byte size
            let byte_offset = start * size_of::<T>();
            let byte_size = (end - start) * size_of::<T>();
            gl::BufferSubData(self._type, byte_offset as isize, byte_size as isize, self.inner.as_ptr() as *const c_void);
            gl::BindBuffer(self._type, 0);
        }
    }
    // Update all
    fn update_all(&self) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferSubData(self._type, 0, (self.inner.len() * size_of::<T>()) as isize, self.inner.as_ptr() as *const c_void);
            gl::BindBuffer(self._type, 0);
        }
    }
}

// Push, set, pop, len
impl<T> DynamicRawBuffer<T> {
    // Set the contents of the dynamic raw buffer from an already allocated vector
    pub fn set_inner(&mut self, vec: Vec<T>) {
        // New reallocation
        let old = self.inner.capacity();
        let new = vec.capacity();
        self.inner = vec;

        if new <= old {
            // If we already have a buffer allocated, no need to reallocate
            self.update_all();
        } else {
            // We must reallocate
            self.reallocate();
        }
    }

    // Push a single element
    pub fn push(&mut self, value: T) {
        let old = self.inner.capacity();
        self.inner.push(value);
        let new = self.inner.capacity();
        let index = self.inner.len() - 1;

        // Reallocate only if we exceed the old capacity
        if new > old {
            self.reallocate();
        } else {
            // Otherwise, just update
            self.update(index..(index + 1))
        }
    }
    // Pop a single element
    pub fn pop(&mut self) {
        self.inner.pop();
    }
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T> GLBufferOperations for DynamicRawBuffer<T> {
    type Data = Vec<T>;

    fn glread(&mut self) -> Result<&Self::Data, OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            // Byte size
            let byte_size = self.inner.len() * size_of::<T>();
            gl::GetBufferSubData(self._type, 0, byte_size as isize, self.inner.as_mut_ptr() as *mut c_void);
            gl::BindBuffer(self._type, 0);
        }
        Ok(&self.inner)
    }
    fn glset(&mut self, data: Self::Data) -> Result<(), OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        self.set_inner(data);
        Ok(())
    }
}

impl<T> Drop for DynamicRawBuffer<T> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        if self.buffer != 0 {
            unsafe {
                gl::DeleteBuffers(1, &mut self.buffer);
            }
        }
    }
}
