use ahash::{AHashMap, AHashSet};
use assets::Assets;
use glutin::{ContextWrapper, PossiblyCurrent, RawContext};

use std::{any::TypeId, collections::HashMap, ptr::null, rc::Rc};
use world::{Storage, World};

use super::get_static_str;
use crate::{
    display::{PrimitiveMode, RasterSettings, Viewport},
    material::{Material, MaterialId},
    shader::Shader,
};

// An abstract wrapper around the whole OpenGL context
pub struct Context {
    // Raw Glutin context
    ctx: RawContext<PossiblyCurrent>,

    // The currently bounded rasterizer settings
    pub(crate) raster: RasterSettings,
    pub(crate) bounded_fbo: u32,
    pub(crate) viewport: Viewport,

    // Reusable shader sources
    pub(crate) stages: (AHashSet<String>, AHashMap<u32, u32>),

    // A list of material surface renderers that we will use
    pipelines: AHashMap<TypeId, Rc<dyn Fn(&mut World)>>,
}

impl Context {
    // Create a context wrapper using a Glutin context
    // This will also enable the default OpenGL settings
    pub(crate) fn new(ctx: ContextWrapper<PossiblyCurrent, ()>) -> Self {
        // Set default OpenGL settings
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
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
            stages: Default::default(),
            pipelines: Default::default(),
        }
    }

    // Register a new pipeline with the specified init settings
    pub fn register_material<'a, M: for<'w> Material<'w>>(
        &mut self,
        storage: &mut Storage<Shader>,
        assets: &mut Assets,
    ) -> MaterialId<M> {
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            let shader = M::shader(self, assets);
            let handle = storage.insert(shader);

            // Main material pipeline
            let closure = move |world: &mut World| {
                crate::pipeline::render_shadows::<M>(world);
                crate::pipeline::render_surfaces::<M>(world, handle.clone());
            };

            self.pipelines.insert(key, Rc::new(closure));
        }
        MaterialId(Default::default())
    }

    // Get a PipeId from a pre-initialized pipeline
    pub fn material_id<M: for<'w> Material<'w>>(&self) -> Option<MaterialId<M>> {
        let key = TypeId::of::<M>();
        self.pipelines
            .get(&key)
            .map(|_| MaterialId(Default::default()))
    }

    // Extract all the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn Fn(&mut World)>> {
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
