use ahash::{AHashMap, AHashSet};
use assets::Assets;
use itertools::Itertools;
use snailquote::unescape;
use std::{
    any::TypeId,
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use thiserror::Error;

use crate::{context::Graphics, prelude::{GpuPod, Texture}, format::Texel};
use super::{ShaderError, ShaderModule, PushConstantLayout, ModuleVisibility};

// This is a compiler that will take GLSL code and create a WGPU module
// This compiler also allows us to define constants and snippets before compilation
// This compiler will be used within the Shader and ComputeShader to compile the modules in batch
pub struct Compiler<'a> {
    pub(crate) assets: &'a Assets,
    pub(crate) graphics: &'a Graphics,
}

impl<'a> Compiler<'a> {
    // Create a new default compiler with the asset loader
    pub fn new(assets: &'a Assets, graphics: &'a Graphics) -> Self {
        Self {
            assets,
            graphics,
        }
    }

    // Convert the given GLSL code to SPIRV code, then compile said SPIRV code
    // This uses the defined resoures defined in this compiler
    pub(crate) fn compile<M: ShaderModule>(&self, module: M) -> Result<Compiled<M>, ShaderError> {
        todo!()
    }
}

impl<'a> Compiler<'a> {
    // Define a uniform buffer type's inner struct type
    pub fn use_uniform_buffer<T: GpuPod>(&mut self, name: impl ToString) {
    }

    // Define a storage buffer type's inner struct type
    pub fn use_storage_buffer<T: GpuPod>(&mut self, name: impl ToString, read: bool, write: bool) {
    }

    // Define a uniform sampled texture's type and texel
    pub fn use_sampled_texture<T: Texture>(&mut self, name: impl ToString, comparison: bool) {
    }

    // Define a uniform sampler's type and texel
    pub fn use_sampler<T: Texel>(&mut self, name: impl ToString, comparison: bool) {
    }

    // Define a storage texture that we can read / write to
    pub fn use_storage_texture<T: Texture>(&mut self, name: impl ToString, read: bool, write: bool) {
    }

    // Define a push constant range to be pushed
    pub fn use_push_constant_layout(&mut self, layout: PushConstantLayout) {
    }
}

// This is a compiled shader module that we can use in multiple pipelines
// We can clone this shader module since we should be able to share them
pub struct Compiled<M: ShaderModule> {
    // Wgpu module and spirv reflected module
    raw: Arc<wgpu::ShaderModule>,
    checksum: u32,

    // Helpers
    path: Arc<Path>,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Clone for Compiled<M> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            path: self.path.clone(),
            _phantom: self._phantom.clone(),
            graphics: self.graphics.clone(),
            checksum: self.checksum,
        }
    }
}

impl<M: ShaderModule> Compiled<M> {
    // Get the raw wgpu hidden module
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.raw
    }

    // Get the visibility of this module
    pub fn visibility(&self) -> ModuleVisibility {
        M::visibility()
    }

    // Get the shader module path for this module
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Get the shader module name for this module
    pub fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}
