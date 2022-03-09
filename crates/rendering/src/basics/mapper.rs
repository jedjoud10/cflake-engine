use std::{ffi::c_void, marker::PhantomData, ptr::{write, write_unaligned}, mem::ManuallyDrop, intrinsics::copy_nonoverlapping};

use gl::types::GLuint;

// A buffer mapper that can write data to an OpenGL buffer
pub struct MappedBufferWriter<'a, Buffer: MappableGLBuffer<Storage>, Storage> {
    _type: GLuint,
    glbuffer: GLuint,
    length: usize,
    caller: &'a mut Buffer,
    storage: PhantomData<&'a mut Storage>,
}

// A buffer mapper that can read data from an OpenGL buffer 
pub struct MappedBufferReader<'a, Buffer: MappableGLBuffer<Storage>, Storage> {
    _type: GLuint,
    buffer: GLuint,
    length: usize,
    caller: &'a Buffer,
    storage: PhantomData<&'a Storage>,
}

// Write
impl<'a, Buffer: MappableGLBuffer<Storage>, Storage> MappedBufferWriter<'a, Buffer, Storage> {
    // New
    pub(crate) fn new(caller: &'a mut Buffer, _type: GLuint, glbuffer: GLuint, length: usize) -> Self {
        Self {
            _type,
            glbuffer,
            length,
            caller,
            storage: Default::default(),
        }
    }
    // Write to the OpenGL buffer directly
    pub fn write(self, value: Storage) {
        unsafe {
            /*
            // Get the mapped OpenGL pointer 
            let ptr = gl::MapBuffer(self._type, gl::MAP_WRITE_BIT);
            // We must not run the drop
            let mut value = ManuallyDrop::new(value);
            // Convert le pointer first
            let converted = Buffer::storage_as_ptr(&value);
            copy_nonoverlapping(converted, ptr, 1);
            unsafe { ManuallyDrop::drop(&mut value) }
            */
        }
    }
    //Store the new value into Self if we implement StorableBuffer, then write to the OpenGL buffer
    pub fn store_then_write(self, value: Storage) where Buffer: StorableGLBuffer<Storage> {
        unsafe {
            /*
            // Get the mapped OpenGL pointer 
            let ptr = gl::MapBuffer(self._type, gl::MAP_WRITE_BIT);
            self.caller.store(value);
            // Convert le pointer first
            let converted = Buffer::storage_as_ptr(self.caller.get_inner_glstorage());
            copy_nonoverlapping(converted, ptr, 1);
            */
        }
    }
}
/*
// Read
impl<'a, Buffer, Storage> MappedBufferReader<'a, Buffer, Storage> {
    // New
    pub(crate) fn new(ptr: *const c_void, length: usize) -> Self {
        Self {
            ptr,
            length,
            _phantom: Default::default(),
            storage: Default::default(),
        }
    }
    // Read from the buffer
    fn read(self) -> &'a Storage {
        todo!()
    }
}
*/


// A mappable buffer that we can write/read to
pub trait MappableGLBuffer<GLStorage> where Self: Sized {
    // Create a MappedBufferReader
    fn map_reader<'a>(&'a self) -> MappedBufferReader<'a, Self, GLStorage>;
    // Create a MappedBufferWriter
    fn map_writer<'a>(&'a mut self) -> MappedBufferWriter<'a, Self, GLStorage>;
}

// A buffer that stores a copy of it's underlying Storage on the Rust side
// This doubles memory usage for said buffer, but we can modify the buffer without having to allocate the Storage again
pub trait StorableGLBuffer<GLStorage> where Self: Sized {
    // Get the underlying stored GLStorage
    fn get_inner_glstorage(&self) -> &GLStorage;
    // Store a new value into the buffer
    fn store(&mut self, value: GLStorage);
}