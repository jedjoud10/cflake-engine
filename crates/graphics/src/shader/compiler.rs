use std::{marker::PhantomData, sync::Arc};
use vulkan::{vk, Device};
use crate::{ModuleKind, Module, Graphics, Processed};

// This is a compiled shader module that we can use in multiple pipelines
pub struct Compiled<M: Module> {
    graphics: Graphics,
    raw: vk::ShaderModule,
    kind: ModuleKind,
    file_name: String,
    spirv: Vec<u32>,
    _phantom: PhantomData<M>,
}

impl<M: Module> Drop for Compiled<M> {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_shader_module(self.raw);
        }
    }
}

impl<M: Module> Compiled<M> {
    // Compile a shader module by using it's processed counter part
    pub fn compile(graphics: &Graphics, module: Processed<M>) -> Self {       
        let kind = module.kind;
        let file_name = module.file_name;
        let source = module.source;
        log::debug!("Created a compiled wrapper for {}", file_name);

        let (spirv, raw) = unsafe {
            let kind = match kind {
                ModuleKind::Vertex => vulkan::ShaderKind::Vertex,
                ModuleKind::Fragment => vulkan::ShaderKind::Fragment,
                ModuleKind::Compute => vulkan::ShaderKind::Compute,
            };

            let spirv = graphics.device().translate_glsl_spirv(&source, &file_name, "main", kind);
            let raw = graphics.device().compile_shader_module(&spirv);
            (spirv, raw)
        };

        Self {
            graphics: graphics.clone(),
            raw,
            kind,
            file_name,
            spirv,
            _phantom: PhantomData,
        }
    }

    // Get the underlying raw Vulkan shader module
    pub fn raw(&self) -> vk::ShaderModule {
        self.raw
    }

    // Get the shader module kind for this compiled shader
    pub fn kind(&self) -> ModuleKind {
        self.kind
    }

    // Get the internal SPIRV representation of the code
    pub fn byte_code(&self) -> &[u32] {
        &self.spirv
    }

    // Get the shader module file name for this module
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}