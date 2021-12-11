// Functions that need to be called on the main thread
pub mod input {
    // Create the key cache at the start of the world initialization
    pub fn create_key_cache() {
        let mut w = crate::world::world_mut();
        w.input_manager.create_key_cache();
    }
}
