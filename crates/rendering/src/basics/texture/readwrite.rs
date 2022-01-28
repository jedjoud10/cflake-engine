use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};

use crate::basics::transfer::{Transfer, Transferable};
// Used to help reading back the bytes from a texture that can be read from
#[derive(Default)]
pub struct TextureReadBytes {
    // The shared bytes that have been sent from the main thread that we must update
    pub(crate) cpu_bytes: Arc<Mutex<Vec<u8>>>,
}

impl TextureReadBytes {
    // Fill a vector of type elements using the appropriate bytes
    pub fn fill_vec<U: Default + Clone>(self) -> Option<Vec<U>> {
        // Read the bytes
        let bytes = Arc::try_unwrap(self.cpu_bytes).ok()?.into_inner().ok()?;
        if bytes.len() == 0 {
            return None;
        }
        // We must now convert the bytes into the vector full of pixels
        let mut clone_test = std::mem::ManuallyDrop::new(bytes);
        let new_len = clone_test.len() / size_of::<U>();
        let vec = unsafe { Vec::from_raw_parts(clone_test.as_mut_ptr() as *mut U, new_len, new_len) };
        Some(vec)
    }
}

impl Transferable for TextureReadBytes {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(Self {
            cpu_bytes: self.cpu_bytes.clone(),
        })
    }
}

// Used to help writing the bytes to a writable texture
#[derive(Default, Clone)]
pub struct TextureWriteBytes {}

impl Transferable for TextureWriteBytes {
    fn transfer(&self) -> Transfer<Self> {
        Transfer(self.clone())
    }
}
