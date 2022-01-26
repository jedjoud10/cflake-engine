use std::sync::{Arc, Mutex};

// Used to help reading back the bytes from a texture that can be read from
pub struct TextureReadBytes {
    // The shared bytes that have been sent from the main thread that we must update
    pub(crate) cpu_bytes: Arc<Mutex<Vec<u8>>>,
}

// Used to help writing the bytes to a writable texture
pub struct TextureWriteBytes {
}