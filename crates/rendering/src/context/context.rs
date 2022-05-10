use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use std::{marker::PhantomData, rc::Rc, collections::HashMap};

// Main cotnext that stores the OpenGL glunit context
#[derive(Clone)]
pub struct Context {
    // Kinda useless for now
    ctx: Rc<RawContext<PossiblyCurrent>>,
    _phantom: PhantomData<*const ()>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx: Rc::new(ctx),
            _phantom: Default::default(),
        }
    }
}
