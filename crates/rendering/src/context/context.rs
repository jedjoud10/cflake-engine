use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use crate::texture;

// Default context values
struct DefaultObjects {
    sampler: texture::Sampler,
}

// Main cotnext that stores the OpenGL glunit context
pub struct Context {
    // Kinda useless for now
    ctx: Rc<RawContext<PossiblyCurrent>>,
    _phantom: PhantomData<*const ()>,

    // Default values
    default: Option<DefaultObjects>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        // TODO: Fix this context init shit
        let mut me = Self {
            ctx: Rc::new(ctx),
            _phantom: Default::default(),
            default: None,
        };

        // Initialize the default values
        let default = DefaultObjects {
            sampler: texture::Sampler::new(texture::Filter::Linear, texture::Wrap::Repeat, &mut me),
        };

        // Overwrite the funny
        me.default = Some(default);
        me
    }

    // Get the default texture sampler
    pub fn sampler(&self) -> &texture::Sampler {
        &self.default.as_ref().unwrap().sampler
    }
}
