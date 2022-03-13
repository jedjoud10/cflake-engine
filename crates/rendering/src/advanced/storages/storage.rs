use crate::utils::{AccessType, UsageType};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of};

// Raw OpenGL storage, just an allocation helper basically
#[derive(Getters, CopyGetters)]
pub struct RawStorage {
    // The OpenGL data for this buffer
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get_copy = "pub")]
    _type: GLuint,

    // Other data
    #[getset(get_copy = "pub")]
    usage: UsageType,

    // IN BYTES
    #[getset(get_copy = "pub")]
    byte_cap: usize,
    #[getset(get_copy = "pub")]
    byte_len: usize,
}

// Creation
impl RawStorage {
    // Create the raw storage, and possibly initialize it
    pub unsafe fn new(cap: usize, len: usize, ptr: *const c_void, _type: u32, usage: UsageType) -> Self {
        let buffer = unsafe {
            let mut buffer = 0;
            gl::GenBuffers(1, &mut buffer);
            buffer
        };
        // If we will allocate the buffer once, make it immutable
        unsafe {
            if usage.dynamic {
                // Can have multiple allocations
                gl::BindBuffer(_type, buffer);
                gl::BufferData(_type, (cap) as isize, ptr as *const c_void, usage.convert());
            } else {
                // Single allocation
                gl::BindBuffer(_type, buffer);
                let bits = match usage.access {
                    AccessType::ClientToServer => gl::DYNAMIC_STORAGE_BIT | gl::MAP_WRITE_BIT,
                    AccessType::ServerToClient => gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT,
                    AccessType::ServerToServer => gl::MAP_READ_BIT,
                };
                gl::BufferStorage(_type, (cap) as isize, ptr as *const c_void, bits);
            }
        }
        Self {
            buffer,
            _type,
            usage,
            byte_cap: cap,
            byte_len: len,
        }
    }
    // Update the buffer
    pub unsafe fn update(&mut self, ptr: *const c_void, cap: usize, len: usize) {
        // Check if we need to reallocate
        self.byte_len = len;
        if cap > self.byte_cap {
            // Check if we can reallocate first
            if !self.usage.dynamic {
                panic!()
            }

            // Reallocate
            self.byte_cap = cap;
            self.reallocate(ptr, cap);
        } else {
            // Update subdata
            self.update_subdata(ptr, len);
        }
    }
    // Completely reallocate
    pub unsafe fn reallocate(&mut self, ptr: *const c_void, cap: usize) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferData(self._type, (cap) as isize, ptr as *const c_void, self.usage.convert());
        }
    }
    // Update subdata
    pub unsafe fn update_subdata(&mut self, input: *const c_void, len: usize) {
        unsafe {
            gl::BindBuffer(self._type, self.buffer);
            gl::BufferSubData(self._type, 0, len as isize, input as *const c_void);
        }
    }
    // Read subdata
    pub unsafe fn read(&self, output: *mut c_void, len: usize, _offset: usize) {
        /*
        // Map the buffer
        let ptr = {
            gl::BindBuffer(self._type, self.buffer);
            let ptr = gl::MapBuffer(self._type, gl::READ_ONLY);
            // Check validity
            if ptr.is_null() {
                panic!()
            }
            ptr
        };
        // Then copy to output
        let i = std::time::Instant::now();
        std::ptr::copy(ptr as *const Element, output, len);

        // We can unmap the buffer now
        let i = std::time::Instant::now();
        let _result = gl::UnmapBuffer(self._type);
        */
        gl::BindBuffer(self._type, self.buffer);
        gl::GetBufferSubData(self._type, 0, (len) as isize, output as *mut c_void);
    }
}

impl Drop for RawStorage {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        unsafe {
            // The buffer should always be valid
            gl::DeleteBuffers(1, &mut self.buffer);
        }
    }
}

// Raw typed OpenGL storage, with a specific type
#[derive(Getters, CopyGetters)]
pub struct TypedStorage<Element> {
    // The OpenGL data for this buffer
    #[getset(get = "pub")]
    raw: RawStorage,
    #[getset(get_copy = "pub")]
    capacity: usize,
    #[getset(get_copy = "pub")]
    len: usize,
    _phantom: PhantomData<*const Element>,
}

// Creation
impl<Element> TypedStorage<Element> {
    // Create the raw storage, and possibly initialize it
    pub unsafe fn new(cap: usize, len: usize, ptr: *const Element, _type: u32, usage: UsageType) -> Self {
        Self {
            raw: RawStorage::new(cap * size_of::<Element>(), len * size_of::<Element>(), ptr as *const c_void, _type, usage),
            _phantom: PhantomData::default(),
            capacity: cap,
            len,
        }
    }
    // Get the OpenGL buffer that backs this buffer
    pub fn buffer(&self) -> GLuint {
        self.raw.buffer
    }
    // Update the buffer
    pub fn update(&mut self, ptr: *const Element, cap: usize, len: usize) {
        // Also update self
        self.capacity = self.capacity.max(cap);
        self.len = len;
        unsafe { self.raw.update(ptr as *const c_void, cap * size_of::<Element>(), len * size_of::<Element>()) }
    }
    // Read subdata
    pub fn read(&self, output: *mut Element, len: usize, offset: usize) {
        // Cannot read more than we have allocated!
        if len > self.capacity {
            panic!()
        }
        unsafe { self.raw.read(output as *mut c_void, len * size_of::<Element>(), offset * size_of::<Element>()) }
    }
}
