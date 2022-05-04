use std::marker::PhantomData;

use glutin::{RawContext, ContextWrapper, PossiblyCurrent};

// Main cotnext that stores the OpenGL glunit context
pub struct Context {
    ctx: RawContext<PossiblyCurrent>,
    _phantom: PhantomData<*const ()>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx,
            _phantom: Default::default(),
        }
    }
}