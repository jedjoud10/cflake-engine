use crate::utils::{AccessType, UsageType};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::size_of};

// Storage that contains a contiguous array of a specific element on the GPU using OpenGL buffers
#[derive(Getters, CopyGetters)]
pub struct Storage<Element> {
    // OpenGL buffer info
    #[getset(get_copy = "pub")]
    buffer: GLuint,
    #[getset(get_copy = "pub")]
    _type: GLuint,

    // How we shall access the data on the GPU side
    #[getset(get_copy = "pub")]
    usage: UsageType,
    
    // Allocation info shiz
    #[getset(get_copy = "pub")]
    capacity: usize,
    #[getset(get_copy = "pub")]
    len: usize,

    // Boo bitch
    _phantom: PhantomData<*const Element>,
}

impl<Element> Storage<Element> {
    // Create a new empty storage
    // TODO: Create an OpenGL context shit thingy
    pub fn new(_type: u32, usage: UsageType) -> Self {
        Self {
            buffer: unsafe {     
                let mut buffer = 0;
                gl::GenBuffers(1, &mut buffer);
                gl::BindBuffer(_type, buffer);
                buffer
            },
            _type,
            usage,
            capacity: 0,
            len: 0,
            _phantom: Default::default(),
        }
    }

    // Create the raw storage, and possibly initialize it
    pub unsafe fn new(cap: usize, len: usize, ptr: *const Element, _type: u32, usage: UsageType) -> Self {
        Self {
            raw: GlBuffer::new(cap * size_of::<Element>(), len * size_of::<Element>(), ptr as *const c_void, _type, usage),
            _phantom: PhantomData::default(),
            capacity: cap,
            len,
        }
    }
    // Get the OpenGL buffer that backs this buffer
    pub fn buffer(&self) -> GLuint {
        self.raw.buffer
    }
    // Update the buffer using another pointer
    pub fn update(&mut self, ptr: *const Element, cap: usize, len: usize) {
        // Also update self
        self.capacity = self.capacity.max(cap);
        self.len = len;
        unsafe { self.raw.update(ptr as *const c_void, cap * size_of::<Element>(), len * size_of::<Element>()) }
    }
    // Read subdata
    pub fn read(&self, output: *mut Element, len: usize, offset: usize) {
        // Cannot read more than we have allocated
        assert!(len < self.len, "Cannot read more than we have");
        unsafe { self.raw.read(output as *mut c_void, len * size_of::<Element>(), offset * size_of::<Element>()) }
    }

    // Create the raw storage, and possibly initialize it
    pub unsafe fn new(cap: usize, len: usize, ptr: *const c_void, _type: u32, usage: UsageType) -> Self {
        
        // If we will allocate the buffer once, make it immutable
        if usage.dynamic {
            // Can have multiple allocations
            gl::NamedBufferData(buffer, (cap) as isize, ptr as *const c_void, usage.convert());
        } else {
            // Single allocation
            let bits = match usage.access {
                AccessType::ClientToServer => gl::DYNAMIC_STORAGE_BIT | gl::MAP_WRITE_BIT,
                AccessType::ServerToClient => gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT,
                AccessType::ServerToServer => gl::MAP_READ_BIT,
            };
            gl::NamedBufferStorage(buffer, (cap) as isize, ptr as *const c_void, bits);
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
            assert!(self.usage.dynamic, "Buffer is not dynamic, cannot reallocate!");

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
        gl::NamedBufferData(self.buffer, (cap) as isize, ptr as *const c_void, self.usage.convert());
    }
    // Update subdata
    pub unsafe fn update_subdata(&mut self, input: *const c_void, len: usize) {
        gl::NamedBufferSubData(self.buffer, 0, len as isize, input as *const c_void);
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
        std::ptr::copy(ptr, output, len);

        // We can unmap the buffer now
        let _result = gl::UnmapBuffer(self._type);
        */
        gl::GetNamedBufferSubData(self.buffer, 0, (len) as isize, output as *mut c_void);
    }
}

impl<T> Drop for Storage<T> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        unsafe {
            // The buffer should always be valid
            gl::DeleteBuffers(1, &mut self.buffer);
        }
    }
}

// Buffer trait
pub trait Buffer
where
    Self: Sized,
{
    // Inner storage element
    type Element;

    // Get the OpenGL buffer that backs this buffer
    fn buffer(&self) -> GLuint;
    // Get the raw Storage
    fn storage(&self) -> &Storage<Self::Element>;
    // Create a new buffer using some capacity, length, and a pointer
    unsafe fn new_raw(cap: usize, len: usize, ptr: *const Self::Element, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self;
    // Create a new empty vector
    fn empty(_type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        unsafe { Self::new_raw(0, 0, null(), _type, usage, _pipeline) }
    }
    // Create a new buffer using a vector
    fn new(vec: Vec<MaybeUninit<Self::Element>>, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        let ptr = vec.as_ptr() as *const Self::Element;
        unsafe { Self::new_raw(vec.capacity(), vec.len(), ptr, _type, usage, _pipeline) }
    }
    // Create a new buffer using a specified capacity
    fn with_capacity(capacity: usize, _type: GLuint, usage: UsageType, _pipeline: &Pipeline) -> Self {
        unsafe { Self::new_raw(capacity, 0, null(), _type, usage, _pipeline) }
    }
    // Get the underlying data
    fn read(&mut self, buf: &mut [Self::Element]);
    // Set the underlying data
    // This will take all the values out of buf and store them into the OpenGL buffer
    fn write(&mut self, buf: &[Self::Element])
    where
        Self::Element: Copy;
}
