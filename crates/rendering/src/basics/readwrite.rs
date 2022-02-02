use std::{
    mem::{size_of, ManuallyDrop},
    sync::{Arc, Mutex},
};

use crate::basics::transfer::{Transfer, Transferable};
// Used to help reading back the bytes from OpenGL storage
#[derive(Default)]
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
        let mut bytes = ManuallyDrop::new(Arc::try_unwrap(self.bytes).ok()?.into_inner().ok()?);
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
        let locked = self.bytes.lock().ok()?;
        let bytes = &*locked;
        let src_ptr = bytes.as_ptr();
        let dst_ptr = arr.as_mut_ptr() as *mut u8;
        let new_len = bytes.len() / size_of::<U>();
        // Check if the byte count is legal
        if byte_count != bytes.len() || bytes.is_empty() { return None }
        unsafe { 
            // Write
            // Does this cause a memory leak? I have no fucking clue.
            std::ptr::copy(src_ptr, dst_ptr, arr.len())
        }
        Some(())
    }
}

impl Transferable for ReadBytes {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(Self {
            bytes: self.bytes.clone(),
            range: None,
        })
    }
}
