use std::sync::{Arc, Mutex};
// Used to help reading back the bytes from a texture that can be read from
#[derive(Default, Clone)]
pub struct TextureReadBytes {
    // The shared bytes that have been sent from the main thread that we must update
    pub(crate) cpu_bytes: Arc<Mutex<Vec<u8>>>,
}

impl TextureReadBytes {
    // Fill a vector of type elements using the appropriate bytes
    pub fn fill_vec<U: Default + Clone>(self) -> Option<Vec<U>>
    {
        // Read the bytes
        let bytes = Arc::try_unwrap(self.cpu_bytes).ok()?.into_inner().ok()?;
        if bytes.len() == 0 { return None; }
        // We must now convert the bytes into the vector full of pixels
        let vec = bytes.chunks_exact(std::mem::size_of::<U>()).map(|x| unsafe { std::ptr::read::<U>(x.as_ptr() as *const _) });
        Some(vec.collect::<Vec<_>>())
    }
}

// Used to help writing the bytes to a writable texture
pub struct TextureWriteBytes {
}