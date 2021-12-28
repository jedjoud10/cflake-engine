use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

// Create a counted ID using an atomic
pub fn get_id() -> u32 {
    static ATOMIC: AtomicU32 = AtomicU32::new(0);
    ATOMIC.fetch_add(1, Relaxed)
}