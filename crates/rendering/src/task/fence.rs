use crate::context::Context;

// A fence is just an asynchronous OpenGL task that we will use to detect when some async task finishes executing
pub struct Fence {
}

impl Fence {
    // Detect if the task finished execution
    pub fn finished(&self, ctx: &Context) -> bool {
        false
    }
}