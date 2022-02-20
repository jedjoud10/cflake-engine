use std::{
    mem::{size_of, ManuallyDrop},
    sync::Arc,
};


use parking_lot::Mutex;
// Used to help reading back the bytes from OpenGL storage
#[derive(Default, Clone)]
pub struct ReadBytes {
    // The shared bytes that have been sent from the main thread that we must update
    pub(crate) bytes: Arc<Mutex<Vec<u8>>>,
    // A specific range of bytes to read from, if we want to
    pub(crate) range: Option<std::ops::Range<usize>>,
}

impl ReadBytes {
    // Create a new read bytes with a specific range
    pub fn with_range(range: std::ops::Range<usize>) -> Self {
        Self {
            range: Some(range),
            ..Default::default()
        }
    }
    // Fill a vector of type elements using the appropriate bytes
    pub fn fill_vec<U>(self) -> Option<Vec<U>> {
        // Read the bytes
        let mut bytes = ManuallyDrop::new(Arc::try_unwrap(self.bytes).ok()?.into_inner());
        if bytes.is_empty() {
            return None;
        }
        // We must now convert the bytes into the vector full of elements
        let new_len = bytes.len() / size_of::<U>();
        let vec = unsafe { Vec::from_raw_parts(bytes.as_mut_ptr() as *mut U, new_len, new_len) };
        Some(vec)
    }
    // Fill an already allocated array
    pub fn fill_array<U>(self, arr: &mut [U]) -> Option<()> {
        let len = arr.len();
        let byte_count = len * size_of::<U>();
        // Read the bytes
        let mut bytes = ManuallyDrop::new(Arc::try_unwrap(self.bytes).ok()?.into_inner());
        let src_ptr = bytes.as_ptr();
        let dst_ptr = arr.as_mut_ptr() as *mut u8;
        let _new_len = bytes.len() / size_of::<U>();
        // Check if the byte count is legal
        if byte_count != bytes.len() || bytes.is_empty() {
            return None;
        }
        unsafe {
            // Write
            // Does this cause a memory leak? I have no fucking clue.
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, byte_count);
            ManuallyDrop::drop(&mut bytes);
        }
        Some(())
    }
}

// Helps writing to some OpenGL buffers
#[derive(Default)]
pub struct WriteBytes {
    // The bytes that we will write to the receiving buffer
    pub(crate) bytes: Vec<u8>,
}

impl WriteBytes {
    // Create some new bytes to write using a vector of structs
    // "T" should have repr(C) just to be safe
    pub fn new<T: Sized>(vec: Vec<T>) -> Self {
        // Transmute
        let mut vec = ManuallyDrop::new(vec);
        let new_len = size_of::<T>() * vec.len();
        let bytes = unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut u8, new_len, new_len) };
        // Now we can drop safely
        Self {
            bytes,
        }
    }
}