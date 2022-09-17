use ahash::AHashMap;
use assets::Assets;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};
use nohash_hasher::NoHashHasher;
use world::Storage;
use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    hash::BuildHasherDefault,
    ptr::null,
    rc::Rc,
    sync::{atomic::AtomicU64, Arc, Mutex},
    time::Duration,
};

use super::get_static_str;
use crate::{
    display::{PrimitiveMode, RasterSettings, Viewport},
    pipeline::{CreatePipeline, PipeId, Pipeline}, material::Material, shader::Shader,
};

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    // Raw Glutin context
    ctx: RawContext<PossiblyCurrent>,

    // The currently bounded rasterizer settings
    pub(crate) raster: RasterSettings,
    pub(crate) bounded_fbo: u32,
    pub(crate) viewport: Viewport,
    
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

        Self {
            ctx,
            raster: RasterSettings {
                depth_test: None,
                scissor_test: None,
                primitive: PrimitiveMode::Triangles { cull: None },
                srgb: false,
                blend: None,
            },
            bounded_fbo: 0,
            viewport: Viewport {
                origin: vek::Vec2::zero(),
                extent: vek::Extent2::zero(),
            },
            pipelines: Default::default(),
        }
    }

    // Register a new pipeline with the specified init settings
    pub fn register_material<'a, M: for<'w> Material<'w>>(
        &mut self,
        storage: &mut Storage<Shader>,
        assets: &mut Assets,
    ) -> MaterialId<P> {
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

    // Flush the context's commands to the driver
    pub fn flush(&mut self) {
        unsafe { gl::Flush() }
    }

    // Force the driver to execute all the commands in the stream
    pub fn finish(&mut self) {
        unsafe {
            gl::Finish();
        }
    }
}
