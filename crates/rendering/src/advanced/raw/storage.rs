use crate::{
    pipeline::Pipeline,
    utils::{AccessType, UsageType, UpdateFrequency, ReallocationType},
};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

// Raw OpenGL storage
#[derive(Getters, CopyGetters)]
pub struct Storage<Element> {
    // The OpenGL data for this buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get_copy = "pub")]
    _type: GLuint,

    // Other data
    #[getset(get_copy = "pub")]
    usage: UsageType,
    _phantom: PhantomData<*const Element>,
    #[getset(get_copy = "pub")]
    capacity: usize,
    #[getset(get_copy = "pub")]
    len: usize,
}

// Creation
impl<Element> Storage<Element> {
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
    // Initialize the raw storage with some empty data
    pub(crate) fn init(&mut self, cap: usize, len: usize, ptr: *const Element) {
        self.len = len;
        self.capacity = cap;
        // If we will allocate the buffer once, make it immutable
        match self.usage.reallocation {
            ReallocationType::StaticallyAllocated => unsafe {
                // Single allocation
                gl::BindBuffer(self._type, self.buffer);
                let bits = gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT;
                gl::BufferStorage(self._type, (cap * size_of::<Element>()) as isize, ptr as *const c_void, bits);
            },
            ReallocationType::DynamicallyAllocated => unsafe {
                // Can have multiple allocations
                gl::BindBuffer(self._type, self.buffer);
                gl::BufferData(self._type, (cap * size_of::<Element>()) as isize, ptr as *const c_void, self.usage.convert());
            },
        }
    }
    // Update the buffer
    pub fn update(&mut self, ptr: *const Element, cap: usize, len: usize) {
        // Check if we need to reallocate
        self.len = len;
        if cap > self.capacity {
            // Check if we can reallocate first
            if let ReallocationType::StaticallyAllocated = self.usage.reallocation { panic!() }

            // Reallocate
            self.capacity = cap;
            self.reallocate(ptr, cap);
        } else {
            // Update subdata
            self.update_subdata(ptr, len);
        }
    }
    // Completely reallocate
    fn reallocate(&mut self, ptr: *const Element, cap: usize) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(self._type, (cap * size_of::<Element>()) as isize, ptr as *const c_void, self.usage.convert());
        }
    }
    // Update subdata
    fn update_subdata(&mut self, ptr: *const Element, len: usize) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferSubData(self._type, 0, (len * size_of::<Element>()) as isize, ptr as *const c_void);
        }
    }
}

impl<Element> Drop for Storage<Element> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        unsafe {
            // The buffer should always be valid
            gl::DeleteBuffers(1, &mut self.buffer);
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
