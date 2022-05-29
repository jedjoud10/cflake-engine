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

use crate::{
    canvas::rasterizer::{FaceCullMode, PrimitiveMode, RasterSettings},
    texture::Bindless,
};

// HashMap that uses the OpenGL types of ojects to keep track of which objects are bound
type BindingHashMap = HashMap<u32, u32, BuildHasherDefault<NoHashHasher<u32>>>;

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    ctx: RawContext<PossiblyCurrent>,
    pub(crate) bound: BindingHashMap,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx,
            bound: Default::default(),
        }
    }

    // This will check if an object of a unique target type is currently bound to the context
    pub(crate) fn is_bound(&self, target: u32, object: u32) -> bool {
        self.bound.get(&target).map(|&bound| bound == object).unwrap_or_default()
    }

    // This will bind an object if it wasn't bound already
    // This will execute the "update" callback whenever we must bind the object
    pub(crate) fn bind(&mut self, target: u32, object: u32, update: impl FnOnce(u32)) {
        // Bind the raw object first
        (!self.is_bound(target, object)).then(|| update(object));

        *self.bound.entry(target).or_insert(object) = object;
    }

    // Get the raw Glutin OpenGL context wrapper
    pub fn raw(&self) -> &RawContext<PossiblyCurrent> {
        &self.ctx
    }
}
