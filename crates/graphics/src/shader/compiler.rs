use crate::{Graphics, ModuleKind, ShaderModule, ShaderCompilationError, FunctionModule, ShaderIncludeError};
use std::{ffi::CStr, marker::PhantomData, time::Instant, path::PathBuf};
use ahash::AHashMap;
use assets::Assets;
use vulkan::{vk, shaderc::{ResolvedInclude, IncludeType, ShaderKind}};


pub struct Compiler<M: ShaderModule> {
    kind: ModuleKind,
    source: String,
    file_name: String,
    snippets: AHashMap<String, String>,
    constants: AHashMap<String, String>,
    _phantom: PhantomData<M>,
}

impl<M: ShaderModule> Compiler<M> {
    // Create a compiler that will execute over the given module
    pub fn new(module: M) -> Self {
        let kind = module.kind();
        let (file_name, source) = module.into_raw_parts();
        log::debug!("Created a new compiler for {}", file_name);

        Self {
            kind,
            source,
            file_name,
            _phantom: PhantomData,
            snippets: Default::default(),
            constants: Default::default(),
        }
    }

    // Include a constant directive that will replace specialization constants (stored internally until compile time)
    pub fn define_constant(
        &mut self,
        _name: impl ToString,
        _value: impl ToString,
    ) {
        todo!()
    }

    // Include a snippet directive that will replace ``#include`` lines that don't refer to a file
    pub fn define_snippet(
        &mut self,
        name: impl ToString,
        value: impl ToString,
    ) {
        let name = name.to_string();
        log::debug!(
            "Defined snippet '{}' for processor '{}'",
            &name,
            &self.file_name
        );
        self.snippets.insert(name, value.to_string());
    }

    // Convert the GLSL code to SPIRV code, then compile said SPIRV code
    pub fn compile(self, assets: &Assets, graphics: &Graphics) -> Result<Compiled<M>, ShaderCompilationError> {
        let Compiler { 
            kind,
            source,
            file_name,
            snippets,
            constants,
            _phantom
        } = self;

        // Callback to use for loading #include directives
        let callback = |target: &str, _type: IncludeType, current: &str, depth: usize| -> Result<ResolvedInclude, String> {
            if matches!(_type, IncludeType::Relative) {
                panic!("Not supported yet");
            }
            
            let include = handle_include(target, assets, &snippets).unwrap();
            Ok(include)
        };

        // Translate the GLSL code to SPIRV compilation artifacts
        let i = Instant::now();
        let artifacts = unsafe {            
            let kind = match kind {
                ModuleKind::Vertex => ShaderKind::Vertex,
                ModuleKind::Fragment => ShaderKind::Fragment,
                ModuleKind::Compute => ShaderKind::Compute,
            };

            graphics.device().translate_glsl_spirv(
                &source, &file_name, "main", kind,
                callback,
            ).map_err(ShaderCompilationError::TranslationError)?
        };
        log::debug!(
            "Took {:?} to translate '{}' to SPIRV",
            i.elapsed(),
            &file_name
        );

        // Fetch the SPIRV byte from the artifacts
        let spirv = artifacts.as_binary_u8();

        // We do a bit of shader reflection
        let reflected_shader_module = spirv_reflect::create_shader_module(spirv)
            .map_err(String::from)
            .map_err(ShaderCompilationError::ReflectionError)?;

        // Compile the SPIRV bytecode
        let i = Instant::now();
        let raw = unsafe {
            let spirv = bytemuck::cast_slice::<u8, u32>(spirv);
            graphics.device().compile_shader_module(spirv)
        };
        log::debug!(
            "Took {:?} to compile '{}' from SPIRV",
            i.elapsed(),
            &file_name
        );
         

        Ok(Compiled {
            raw,
            kind,
            file_name,
            _phantom,
            graphics: graphics.clone(),
        })
    }
}

// Handle dealing with the include directive (that works with asset paths and snippets)
fn handle_include(
    target: &str,
    assets: &Assets,
    snippets: &AHashMap<String, String>,
) -> Result<ResolvedInclude, ShaderIncludeError> {
    // Either load it as an asset or a snippet
    if resembles_asset_path(&target) {
        load_function_module(target, assets)
    } else {
        load_snippet(target, snippets)
    }
}

// Check if an include directive resembles like an asset path instead of a snippet
fn resembles_asset_path(path: &str) -> bool {
    let value = || {
        let pathbuf = PathBuf::try_from(path).ok()?;
        let extension = pathbuf.extension()?.to_str()?;
        Some(extension == "glsl")
    };
    value().unwrap_or_default()
}

// Load a function module and write it to the output line
fn load_function_module(
    path: &str,
    assets: &Assets,
) -> Result<ResolvedInclude, ShaderIncludeError> {
    // Make sure the path is something we can load (.glsl file)
    let pathbuf = PathBuf::try_from(path).unwrap();

    // Load the path from the asset manager
    let resolved_name = pathbuf.clone().into_os_string().into_string().unwrap();
    let path = pathbuf.as_os_str().to_str().unwrap();
    let content = assets
        .load::<FunctionModule>(path)
        .map(|x| x.source)
        .map_err(ShaderIncludeError::FileAssetError)?;
    Ok(ResolvedInclude {
        resolved_name,
        content,
    })
}

// Load a snippet from the snippets and write it to the output line
fn load_snippet(
    name: &str,
    snippets: &AHashMap<String, String>,
) -> Result<ResolvedInclude, ShaderIncludeError> {
    let snippet = snippets.get(name).ok_or(ShaderIncludeError::SnippetNotDefined)?;
    Ok(ResolvedInclude {
        resolved_name: name.to_string(),
        content: snippet.clone(),
    })
}

// This is a compiled shader module that we can use in multiple pipelines
pub struct Compiled<M: ShaderModule> {
    // Vulkan related data
    raw: vk::ShaderModule,
    kind: ModuleKind,

    // Helpers
    file_name: String,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Drop for Compiled<M> {
    fn drop(&mut self) {
        unsafe {
            self.graphics.device().destroy_shader_module(self.raw);
        }
    }
}

impl<M: ShaderModule> Compiled<M> {
    // Get the underlying raw Vulkan shader module
    pub fn raw(&self) -> vk::ShaderModule {
        self.raw
    }

    // Get the shader module kind for this compiled shader
    pub fn kind(&self) -> ModuleKind {
        self.kind
    }

    // Get the shader module file name for this module
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    // Get the compiled description
    pub fn description(&self) -> CompiledDescription {
        CompiledDescription {
            entry: unsafe {
                CStr::from_bytes_with_nul_unchecked(b"main\0")
            },
            flags: vk::PipelineShaderStageCreateFlags::default(),
            kind: self.kind,
            module: &self.raw,
        }
    }
}

// A description of a compiled shader module that we can use within a pipeline
pub struct CompiledDescription<'a> {
    pub(crate) entry: &'static CStr,
    pub(crate) flags: vk::PipelineShaderStageCreateFlags,
    pub(crate) kind: ModuleKind,
    pub(crate) module: &'a vk::ShaderModule,
}
