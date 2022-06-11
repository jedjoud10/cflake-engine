use ahash::AHashMap;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use std::{
    any::TypeId,
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
    material::{BatchRenderer, Material, MaterialRenderer},
    texture::Bindless,
};

// HashMap that uses the OpenGL types of ojects to keep track of which objects are bound
type BindingHashMap = HashMap<u32, u32, BuildHasherDefault<NoHashHasher<u32>>>;

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    // Raw Glutin context
    ctx: RawContext<PossiblyCurrent>,

    // The currently bound objects (since OpenGL uses a state machine)
    pub(crate) bound: BindingHashMap,

    // A list of material surface renderers that we will use
    renderers: AHashMap<TypeId, Rc<dyn MaterialRenderer>>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        Self {
            ctx,
            bound: Default::default(),
            renderers: Default::default(),
        }
    }

    // This will check if an object of a unique target type is currently bound to the context
    pub(crate) fn is_bound(&self, target: u32, object: u32) -> bool {
        self.bound
            .get(&target)
            .map(|&bound| bound == object)
            .unwrap_or_default()
    }

    // This will bind an object if it wasn't bound already
    // This will execute the "update" callback whenever we must bind the object
    pub(crate) fn bind(&mut self, target: u32, object: u32, update: impl FnOnce(u32)) {
        // Bind the raw object first
        (!self.is_bound(target, object)).then(|| update(object));

        *self.bound.entry(target).or_insert(object) = object;
    }

    // Try to register a material renderer with a callback
    // We use callback since we need to register the renderer only once, and it would be a waste to create it multiple times
    pub(crate) fn register_material_renderer<M: Material, F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Context) -> M::Renderer,
    {
        // Material renderers are defined by their material type, so we can only have one material renderer per material type
        let key = TypeId::of::<M>();

        // Only register the renderer once
        if !self.renderers.contains_key(&key) {
            // Create the RC and call the callback
            let renderer = Rc::new(callback(self));

            // Insert the renderer into the context
            self.renderers.insert(key, renderer);
        }
    }

    // Clone all the material renderers outside the context
    // This is going to be executed every frame, but the number of unique material types is low so we shall'n't worry about it
    pub(crate) fn extract_material_renderer(&self) -> Vec<Rc<dyn MaterialRenderer>> {
        self.renderers
            .iter()
            .map(|(key, value)| value.clone())
            .collect::<_>()
    }

    // Get the raw Glutin OpenGL context wrapper
    pub fn raw(&self) -> &RawContext<PossiblyCurrent> {
        &self.ctx
    }
}
