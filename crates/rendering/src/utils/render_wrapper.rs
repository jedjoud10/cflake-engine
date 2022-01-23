use std::{sync::atomic::AtomicPtr, ptr::null_mut};

// A render wrapper that we can use to share around the glfw context and window context
pub struct RenderWrapper(pub AtomicPtr<glfw::Glfw>, pub AtomicPtr<glfw::Window>);

impl Default for RenderWrapper {
    fn default() -> Self {
        Self (AtomicPtr::new(null_mut()), AtomicPtr::new(null_mut()))
    }
}