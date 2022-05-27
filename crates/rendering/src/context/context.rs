use ahash::AHashMap;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::BuildHasherDefault,
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU64},
    rc::Rc,
    time::{Duration, Instant},
};

use crate::texture::Bindless;

// HashMap that uses the OpenGL types of ojects to keep track of which objects are bound
type BindingHashMap = HashMap<u32, u32, BuildHasherDefault<NoHashHasher<u32>>>;

// Main cotnext that stores the OpenGL glunit context
#[derive(Clone)]
pub struct Context {
    // Kinda useless for now
    ctx: Rc<RawContext<PossiblyCurrent>>,
    _phantom: PhantomData<*const ()>,

    // The current rendering frame
    frame: u128,

    // A list of bindless textures that are currently active
    pub(crate) bindless: Vec<Rc<Bindless>>,

    // A list of objects that are currently bound
    pub(crate) bound: BindingHashMap,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx: Rc::new(ctx),
            _phantom: Default::default(),
            frame: 0,
            bindless: Default::default(),
            bound: Default::default(),
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

    // This will check if an object of a unique target type is currently bound to the context
    pub(crate) fn is_bound(&self, target: u32, object: u32) -> bool {
        self.bound.get(&target).map(|&bound| bound == object).unwrap_or_default()
    }

    // This will bind an object if it wasn't bound already
    // This will return Some(()) if the object was successfully bound, and None when the object was already bound
    pub(crate) fn bind(&mut self, raw: unsafe fn(u32, u32), target: u32, object: u32) -> Option<()> {
        let bound = self.bound.entry(target).or_insert_with(|| unsafe {
            // If this is a totally new object, then force the first bind
            raw(target, object);
            object
        });

        // Check if the currently bound object must be rebound or not
        (*bound != object).then(|| unsafe {
            // We must rebind the object
            raw(target, object);
            *bound = object;
        })
    }
}
