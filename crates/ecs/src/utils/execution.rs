use std::sync::atomic::{AtomicI32, Ordering};

// Current event execution order index
static ORDER_INDEX: AtomicI32 = AtomicI32::new(0);
pub struct EventExecutionOrder;

impl EventExecutionOrder {
    // Sets the global order index
    pub fn set(num: i32) {
        ORDER_INDEX.store(num, Ordering::Relaxed)
    }
    // Reads the global order index, increments it, then sets it
    pub fn fetch_add() -> i32 {
        ORDER_INDEX.fetch_add(1, Ordering::Relaxed)
    }
}
