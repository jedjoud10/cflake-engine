use std::sync::atomic::{AtomicU64, Ordering};

// Create a randomized name with a prefix
static RANDOMIZED_NAME_ID: AtomicU64 = AtomicU64::new(0);
pub fn rname(prefix: &str) -> String {
    // Use the others::id_counter to create a counted ID that we can transform into a String
    let name = format!("{:x}", RANDOMIZED_NAME_ID.fetch_add(1, Ordering::Relaxed));
    format!("{}_{}", prefix, name)
}