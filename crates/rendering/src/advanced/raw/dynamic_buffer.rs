use crate::{pipeline::Pipeline, utils::UsageType, basics::bufop::{Writable, Readable}, object::OpenGLObjectNotInitialized};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ops::Range, ptr::null};

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

// Creation
impl<T> DynamicRawBuffer<T> {
    // Create the dynamic raw buffer
    pub fn new(_type: u32, usage: UsageType, _pipeline: &Pipeline) -> Self {
        Self::with_capacity(_type, 0, usage, _pipeline)
    }
    // Create a new dynamic raw buffer with a specified capacity
    pub fn with_capacity(_type: u32, capacity: usize, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let vec = Vec::<T>::with_capacity(capacity);
        let oid = unsafe {
            let mut oid = 0;
            gl::GenBuffers(1, &mut oid);
            gl::BindBuffer(_type, oid);
            if capacity != 0 {
                gl::BufferData(_type, (size_of::<T>() * capacity) as isize, null(), usage.convert());
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
    pub fn set_inner(&mut self, vec: &[T]) where T: Clone {
        // Completely reallocate
        let old = self.inner.capacity();
        self.inner.clear();
        self.inner.extend_from_slice(vec);
        let new = self.inner.capacity();       
        
        // Reallocate only if we exceed the old capacity
        if new > old {
            self.reallocate();
        } else {
            // Otherwise, just update
            self.update_all()
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

impl<T> Writable for DynamicRawBuffer<T> {
    fn glupdate(&mut self) -> Result<(), OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        self.update_all();
        Ok(())
    }
}

impl<T> Readable for DynamicRawBuffer<T> {
    type Data = Vec<T>;

    fn glread(&mut self) -> Result<&Self::Data, OpenGLObjectNotInitialized> {
        // Check validity
        if self.buffer == 0 {
            return Err(OpenGLObjectNotInitialized);
        }
        Ok(&self.inner)
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
