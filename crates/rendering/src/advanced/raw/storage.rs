use crate::{
    pipeline::Pipeline,
    utils::{AccessType, UsageType},
};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of};

// Raw OpenGL storage
#[derive(Getters, CopyGetters)]
pub struct Storage<E> {
    // The OpenGL data for this buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get_copy = "pub")]
    _type: GLuint,

    // Other data
    #[getset(get_copy = "pub")]
    usage: UsageType,
    _phantom: PhantomData<*const E>,
    #[getset(get_copy = "pub")]
    capacity: usize,
    #[getset(get_copy = "pub")]
    len: usize,
}

// Creation
impl<E> Storage<E> {
    // Create the raw storage
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
            _phantom: PhantomData::default(),
            capacity: 0,
            len: 0,
        }
    }
    // Update the buffer
    pub(crate) fn update(&mut self, vec: &Vec<E>) {
        // Check if we need to reallocate
        if vec.capacity() > self.capacity {
            // Completely reallocate
            self.reallocate(vec);
        } else {
            // Update subdata
            self.update_subdata(vec);
        }
        self.len = vec.len();
    }
    // Completely reallocate
    pub(crate) fn reallocate(&mut self, vec: &Vec<E>) {
        self.capacity = vec.capacity();
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(self._type, (size_of::<E>() * vec.capacity()) as isize, vec.as_ptr() as *const c_void, self.usage.convert());
            gl::BindBuffer(self._type, 0);
        }
    }
    // Update subdata
    pub(crate) fn update_subdata(&mut self, vec: &Vec<E>) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            let byte_size = vec.len() * size_of::<E>();
            gl::BufferSubData(self._type, 0, byte_size as isize, vec.as_ptr() as *const c_void);
            gl::BindBuffer(self._type, 0);
        }
    }
}

impl<E> Drop for Storage<E> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        if self.buffer != 0 {
            unsafe {
                gl::DeleteBuffers(1, &mut self.buffer);
            }
        }
    }
}

/*
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
*/
