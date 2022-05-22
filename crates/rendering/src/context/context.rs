use crate::texture::Bindless;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::BuildHasherDefault,
    marker::PhantomData,
    num::NonZeroU64,
    rc::Rc,
    time::{Duration, Instant},
};
// Main cotnext that stores the OpenGL glunit context
#[derive(Clone)]
pub struct Context {
    // Kinda useless for now
    ctx: Rc<RawContext<PossiblyCurrent>>,
    _phantom: PhantomData<*const ()>,

    // The current rendering frame
    frame: u128,

    // A list of bindless textures that are currently active
    pub(crate) bindless: Vec<(Rc<Bindless>)>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx: Rc::new(ctx),
            _phantom: Default::default(),
            frame: 0,
            bindless: Default::default(),
        }
    }

    // Handle the residency states for all the currently active bindless textures
    fn update_bindless_textures(&mut self) {
        // Remove the RCs of bindless textures that are no longer available
        self.bindless.retain(|rc| Rc::strong_count(rc) > 1);

        // Get the current time
        let now = Instant::now();

        // Convert resident handles to non-resident handles if they timeout
        self.bindless
            .iter()
            .filter(|bindless| {
                // Check if it lived longer than last and if the texture is resident
                let next = bindless.last_shader_usage() + bindless.timeout();

                // Check both requirements
                now >= next && bindless.is_resident()
            })
            .for_each(|bindless| {
                // Make the texture non-resident
                unsafe { gl::MakeTextureHandleNonResidentARB(bindless.handle()) };
                bindless.resident.set(false);
            });
    }

    // This shall be called at the end of every frame
    pub(crate) fn step(&mut self) {
        // Increment current frame count
        self.frame = self.frame.checked_add(1).unwrap();

        // Handle object states
        self.update_bindless_textures();
    }
}
