use std::sync::atomic::{AtomicI32, Ordering};

// System execution order
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SystemExecutionOrder(pub i32);

impl SystemExecutionOrder {
    // Set global
    pub fn set(val: i32) {
        GLOBAL_SYSTEM_EXECUTION_ORDER.store(val, Ordering::Relaxed);
    }
}

// Global execution order. Increments by one each time
pub(crate) static GLOBAL_SYSTEM_EXECUTION_ORDER: AtomicI32 = AtomicI32::new(0);

impl Default for SystemExecutionOrder {
    fn default() -> Self {
        Self(GLOBAL_SYSTEM_EXECUTION_ORDER.fetch_add(1, Ordering::Relaxed))
    }
}
