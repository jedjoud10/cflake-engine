use ahash::AHashMap;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use std::{any::TypeId, cell::RefCell, collections::HashMap, hash::BuildHasherDefault, rc::Rc, time::Duration, ptr::null};

use crate::{
    pipeline::{CreatePipeline, PipeId, Pipeline},
    prelude::{RawFramebuffer},
};
use super::get_static_str;

// HashMap that contains framebuffers and their layout identifiers
pub(crate) type FramebufferHashMap = HashMap<u64, RawFramebuffer, BuildHasherDefault<NoHashHasher<u64>>>;

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    // Raw Glutin context
    ctx: RawContext<PossiblyCurrent>,

    // List of generated framebuffers and their layout hashes
    pub(crate) framebuffers: FramebufferHashMap,

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
            framebuffers: Default::default(),
            pipelines: Default::default(),
        }
    }

    // Register a new pipeline with the specified init settings
    pub fn init_pipe_id<'a, P: Pipeline + CreatePipeline<'a>>(
        &mut self,
        init: &mut P::Args,
    ) -> PipeId<P> {
        let key = TypeId::of::<P>();
        if !self.pipelines.contains_key(&key) {
            let pipeline: Rc<dyn Pipeline> = Rc::new(P::init(self, init));
            self.pipelines.insert(key, pipeline);
        }
        PipeId(Default::default())
    }

    // Get a PipeId from a pre-initialized pipeline
    pub fn get_pipe_id<P: Pipeline>(&self) -> Option<PipeId<P>> {
        let key = TypeId::of::<P>();
        self.pipelines.get(&key).map(|_| PipeId(Default::default()))
    }

    // Extract all the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn Pipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }

    // This is a method called at the end of every frame to release temporary values like binldess textures and framebuffers
    pub(crate) fn release_temporary(&mut self, passed: Duration) {
        self.framebuffers.retain(|_, raw| {
            match raw.current_countdown.checked_sub(passed) {
                Some(val) => { raw.current_countdown = val; true },
                None => false,
            } 
        });
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
