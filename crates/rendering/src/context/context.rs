use ahash::AHashMap;
use assets::Assets;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use std::{any::TypeId, collections::HashMap, hash::BuildHasherDefault, ptr::null, rc::Rc};
use world::{Resource, Storage};

use crate::{
    material::{Material, PipeId, Pipeline, SpecializedPipeline},
    prelude::Shader,
};

use super::get_static_str;

// HashMap that uses the OpenGL types of ojects to keep track of which objects are bound
type BindingHashMap = HashMap<u32, u32, BuildHasherDefault<NoHashHasher<u32>>>;

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    // Raw Glutin context
    ctx: RawContext<PossiblyCurrent>,

    // The currently bound objects (since OpenGL uses a state machine)
    pub(crate) bound: BindingHashMap,

    // A list of material surface renderers that we will use
    pipelines: AHashMap<TypeId, Rc<dyn Pipeline>>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    // This will also enable the default OpenGL settings
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        // Set default OpenGL settings
        unsafe {
            // Always have OpenGL debugging enabled
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(super::callback), null());
        }

        // Create el safe wrapper
        Self {
            ctx,
            bound: Default::default(),
            pipelines: Default::default(),
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

    // Register a specialized material pipeline if it is missing, and return it's specific PipeId
    pub fn pipeline<M: for<'w> Material<'w>>(
        &mut self,
        shaders: &mut Storage<Shader>,
        assets: &mut Assets,
    ) -> PipeId<M> {
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            let pipeline: Rc<dyn Pipeline> = Rc::new(SpecializedPipeline::<M> {
                shader: shaders.insert(M::shader(self, assets)),
                _phantom: Default::default(),
            });
            self.pipelines.insert(key, pipeline);
        }

        PipeId(Default::default())
    }

    // Register a specific pipeline if it is missing an return it's specific PipeID
    pub fn register_pipeline_lazy<P: Pipeline>(&mut self, callback: impl FnOnce() -> P) -> PipeId<>

    // Extract all the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn Pipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }

    // Get the raw Glutin OpenGL context wrapper
    pub fn raw(&self) -> &RawContext<PossiblyCurrent> {
        &self.ctx
    }

    // Get the OpenGL version that we are currently using
    pub fn gl_version(&self) -> &'static str {
        unsafe { get_static_str(gl::VERSION) }
    }

    // Get the GLSL version that we shall use
    pub fn glsl_version(&self) -> &'static str {
        unsafe { get_static_str(gl::SHADING_LANGUAGE_VERSION) }
    }
}
