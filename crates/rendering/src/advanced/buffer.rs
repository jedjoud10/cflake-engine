use crate::utils::{AccessType, BufferHints};
use getset::{CopyGetters, Getters};
use gl::types::GLuint;
use std::{ffi::c_void, marker::PhantomData, mem::{size_of, ManuallyDrop, MaybeUninit}, ptr::null, ops::Range, alloc::Layout};

// Storage that contains a contiguous array of a specific value on the GPU using OpenGL buffers
pub struct Buffer<T> {
    // OpenGL buffer info
    buffer: GLuint,
    target: GLuint,

    // How we shall access the data on the GPU side
    hints: BufferHints,
    
    // Allocation info shiz
    capacity: usize,
    length: usize,

    // Boo bitch
    _phantom: PhantomData<*const T>,
}

impl<T> Buffer<T> {
    // Create a new storage from it's raw parts    
    // TODO: Create an OpenGL context shit thingy
    pub unsafe fn from_raw_parts(target: u32, hints: BufferHints, capacity: usize, len: usize, ptr: *const T) -> Self {
        Self {
            buffer: {
                // We must always create the OpenGL buffer
                let mut buffer = 0;
                gl::GenBuffers(1, &mut buffer);
                gl::BindBuffer(target, buffer);
                
                // Initialize it with the data if needed
                if capacity > 0 {
                    // Convert value-cap to byte-cap
                    let cap = isize::try_from(size_of::<T>() * capacity).unwrap();
                    // Can we resize our storage buffer after we've initialized it?
                    if hints.dynamic {
                        // Allocate bytes for the buffer
                        gl::NamedBufferData(buffer, cap, ptr as *const c_void, hints.into_access_hints());
                    } else {
                        // Single allocation that will stay the same throughout the buffer's lifetime
                        gl::NamedBufferStorage(buffer, cap, ptr as *const c_void, hints.into_mapped_buffer_hints());
                    }
                }
            },
            target: target,
            hints,
            capacity,
            length: len,
            _phantom: Default::default(),
        }
    }

    // Create a new empty storage
    pub fn new(target: u32, hints: BufferHints) -> Self {
        unsafe {
            Self::from_raw_parts(target, hints, 0, 0, null())
        }
    }

    // Create a storage from a vector that is already initialized with some data
    pub fn from_vec(target: u32, hints: BufferHints, vec: Vec<T>) -> Self {
        unsafe {
            // Just to make sure the compiler doesn't drop this vec earlier
            let mut vec = ManuallyDrop::new(vec);
            
            // Oui
            let me = Self::from_raw_parts(target, hints, vec.len(), vec.len(), vec.as_ptr());

            // We can now safely drop the vector, since the data's been sent to the GPU
            ManuallyDrop::drop(&mut vec);
            me
        }
    }
    
    // Get the name of the underlying OpenGL buffer
    pub fn buffer(&self) -> GLuint {
        self.buffer
    }

    // Get the target type of the underlying OpenGL buffer
    pub fn target(&self) -> GLuint {
        self.target
    }

    // Update the buffer using another pointer
    pub fn update(&mut self, ptr: *const T, cap: usize, len: usize) {
        // Also update self
        self.capacity = self.capacity.max(cap);
        self.length = len;
        unsafe { self.raw.update(ptr as *const c_void, cap * size_of::<T>(), len * size_of::<T>()) }
    }

    // Read a specific part of the vector and write it to a pointer
    pub unsafe fn read_into(&self, output: *mut T, offset: usize, length: usize) {
        // Make sure the range can fit within our allocated space
        assert!(offset + length < self.length, "Given range is too large");
        
        // Read from the OpenGL buffer
        let offset = isize::try_from(offset * size_of::<T>()).unwrap();
        let length = isize::try_from(length * size_of::<T>()).unwrap();
        gl::GetNamedBufferSubData(self.buffer, offset, length, output as *mut c_void);
    }

    // Update a part of the buffer using a pointer to some data and a range
    pub unsafe fn write_from(&mut self, input: *const T, offset: usize, length: usize) {
        // Make sure the range can fit within our allocated space
        assert!(offset + length < self.length, "Given range is too large");

        // Write to the OpenGL buffer
        let offset = isize::try_from(offset * size_of::<T>()).unwrap();
        let length = isize::try_from(length * size_of::<T>()).unwrap();
        gl::NamedBufferSubData(self.buffer, offset, length, input as *const c_void);
    }

    // Reallocate the buffer completely using a new pointer
    pub unsafe fn reallocate(&mut self, input: *const T, capacity: usize) {
        // We cannot reallocate if the buffer isn't dynamic
        assert!(!self.hints.dynamic, "Cannot reallocate");

        let capacity = isize::try_from(capacity * size_of::<T>()).unwrap();
        gl::NamedBufferData(self.buffer, capacity, input as *const c_void, self.hints.into_access_hints());
    }

    // Push a new value into the buffer
    pub fn push(&mut self, value: T) {
        // Check if we have enough space allocate to store the value
        if self.capacity == self.length {
            // We shall reallocate the buffer with a new capacity
            let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };
            unsafe {
                // Read from the old buffer and write it to the new buffer
                let layout = Layout::array(new_capacity);
                let ptr = std::alloc::alloc(layout) as *mut T;
                self.read_into(ptr, 0, self.length);

                // Write the new element
                std::ptr::write(ptr.add(self.length), value);                

                // And reallocate
                self.capacity = new_capacity;
                self.reallocate(ptr, self.capacity);

                // We can drop the temporary vector ofc
                std::alloc::dealloc(ptr, layout);
            }
        } else {
            // We are bing-chilling
            unsafe { self.write_from(&value, self.length, 1); }
        }

        // Update the length
        self.length += 1;

    }

    // Pop the last value from the buffer
    pub fn pop(&mut self) -> Option<T> {
        if self.length != 0 {
            self.length -= 1;

            // Read the element from the OpenGL buffer (TODO: Fix)
            unsafe { 
                let val = MaybeUninit::<T>::uninit();
                self.read_into(val.as_mut_ptr(), self.length, 1);
                Some(val.assume_init())
            }
        } else {
            // No elements available
            None
        }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffer
        unsafe {
            // The buffer should always be valid
            gl::DeleteBuffers(1, &mut self.buffer);
        }
    }
}