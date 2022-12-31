use crate::{Graphics, ModuleKind, Processed, ShaderModule};
use std::{marker::PhantomData, sync::Arc, time::Instant};
use vulkan::{vk, Device};

// This is a compiled shader module that we can use in multiple pipelines
pub struct Compiled<M: ShaderModule> {
    graphics: Graphics,
    raw: vk::ShaderModule,
    kind: ModuleKind,
    file_name: String,
    spirv: Vec<u32>,
    _phantom: PhantomData<M>,
}

impl<M: ShaderModule> Drop for Compiled<M> {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_shader_module(self.raw);
        }
    }
}

impl<M: ShaderModule> Compiled<M> {
    // Compile a shader module by using it's processed counter part
    pub fn compile(
        graphics: &Graphics,
        module: Processed<M>,
    ) -> Self {
        let kind = module.kind;
        let file_name = module.file_name;
        let source = module.source;
        log::debug!("Created a compiled wrapper for {}", file_name);

        // Translate the shader to SPIRV and compile it
        let (spirv, raw) = unsafe {
            let kind = match kind {
                ModuleKind::Vertex => vulkan::ShaderKind::Vertex,
                ModuleKind::Fragment => vulkan::ShaderKind::Fragment,
                ModuleKind::Compute => vulkan::ShaderKind::Compute,
            };

            // Translate the GLSL code to SPIRV bytecode
            let i = Instant::now();
            let spirv = graphics.device().translate_glsl_spirv(
                &source, &file_name, "main", kind,
            );
            log::debug!(
                "Took {:?} to translate '{}' to SPIRV",
                i.elapsed(),
                &file_name
            );

            // Compile the SPIRV bytecode
            let i = Instant::now();
            let raw = graphics.device().compile_shader_module(&spirv);
            log::debug!(
                "Took {:?} to compile '{}' from SPIRV",
                i.elapsed(),
                &file_name
            );
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

    // Get the compiled description
    pub fn description(&self) -> CompiledDescription {
        CompiledDescription {
            flags: vk::PipelineShaderStageCreateFlags::default(),
            kind: self.kind,
            module: &self.raw,
        }
    }
}

// A description of a compiled shader module that we can use within a pipeline
// TODO: Remove tis
pub struct CompiledDescription<'a> {
    pub(crate) flags: vk::PipelineShaderStageCreateFlags,
    pub(crate) kind: ModuleKind,
    pub(crate) module: &'a vk::ShaderModule,
}
