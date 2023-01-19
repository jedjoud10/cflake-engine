use crate::{Graphics, ModuleKind, ShaderModule, ShaderCompilationError, FunctionModule, ShaderIncludeError, GpuPodRelaxed, Reflected};
use std::{ffi::CStr, marker::PhantomData, time::Instant, path::PathBuf, any::TypeId};
use ahash::AHashMap;
use assets::Assets;
use vulkan::{vk, shaderc::{ResolvedInclude, IncludeType, ShaderKind, CompilationArtifact}};

// This is a compiler that will take was GLSL code, convert it to SPIRV,
// then to an appropriate Vulkan shader module.
// This compiler also allows us to define constants and snippets before compilation
pub struct Compiler<M: ShaderModule> {
    // Needed for shaderc and Vulkan pipeline module
    kind: ModuleKind,
    source: String,
    file_name: String,

    // Definitions
    snippets: AHashMap<String, String>,
    constants: AHashMap<u32, Vec<u8>>,

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
    // TODO: Make dis work with bool pwease??
    pub fn define_constant<T: GpuPodRelaxed>(
        &mut self,
        id: u32,
        value: T,
    ) {
        let value = [value];
        let slice = bytemuck::cast_slice::<T, u8>(&value);

        self.constants.insert(id, slice.to_owned());
    }

    // Include a snippet directive that will replace ``#include`` lines that don't refer to a file
    pub fn define_snippet(
        &mut self,
        name: impl ToString,
        value: impl ToString,
    ) {
        let name = name.to_string();
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

        // Create the constants specialization info
        let constants = create_constants_wrapper(constants);
        
        // Translate the GLSL code to SPIRV compilation artifacts
        let artifacts = translate_glsl_to_spirv(
            graphics,
            assets,
            &snippets,
            kind,
            &source,
            &file_name
        )?;
        
        // Compile the SPIRV bytecode
        let raw = compile_spirv(
            &artifacts,
            graphics,
            &file_name
        );

        // Reflect the SPIRV bytecode
        let reflected = reflect_spirv(&artifacts);

        Ok(Compiled {
            raw,
            kind,
            file_name,
            _phantom,
            constants,
            reflected,
            graphics: graphics.clone(),
        })
    }
}

// Data that must be stored within the compiled shader
// that indicates how constants are defined in the specialization info
pub struct Constants {
    pub(crate) raw: vk::SpecializationInfo,
    data: Vec<u8>,
    entries: Vec<vk::SpecializationMapEntry>,
}

// Calculate the specialization info based on a hashmap of constants
fn create_constants_wrapper(constants: AHashMap<u32, Vec<u8>>) -> Constants {
    let merged = constants.iter().collect::<Vec<_>>();
    let data = merged.iter().flat_map(|(_, data)| data.iter().cloned()).collect::<Vec<_>>();
    let ids = merged.iter().map(|(id, _)| **id).collect::<Vec<_>>();

    let mut summed_offset = 0;
    let ranges = merged.iter().map(|(_, data)| {
        let offset = summed_offset;
        summed_offset += data.len();
        (offset, offset + data.len())
    });

    let entries: Vec<vk::SpecializationMapEntry> = ids
        .iter()
        .zip(ranges)
        .map(|(id, (start, end))| {
            let size = end - start;
            *vk::SpecializationMapEntry::builder()
                .constant_id(*id)
                .offset(start as u32)
                .size(size)
        })
        .collect::<Vec<_>>();

    let raw = *vk::SpecializationInfo::builder()
        .map_entries(&entries)
        .data(&data);

    Constants {
        raw,
        data,
        entries,
    }
}

// Handle dealing with the include directive (that works with asset paths and snippets)
fn handle_include(
    current: &str,
    _type: IncludeType, 
    target: &str,
    depth: usize,
    assets: &Assets,
    snippets: &AHashMap<String, String>,
) -> Result<ResolvedInclude, ShaderIncludeError> {
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

    // Relative paths not supported yet
    assert!(matches!(_type, IncludeType::Standard), "Not supported yet");

    // Either load it as an asset or a snippet
    if resembles_asset_path(&target) {
        load_function_module(target, assets)
    } else {
        load_snippet(target, snippets)
    }
}

// Translate the GLSL code to SPIRV and handle the includes and such
fn translate_glsl_to_spirv(
    graphics: &Graphics,
    assets: &Assets,
    snippets: &AHashMap<String, String>,
    kind: ModuleKind,
    source: &str,
    file_name: &str,
) -> Result<CompilationArtifact, ShaderCompilationError> {
    let i = Instant::now();
    let artifacts = unsafe {            
        let kind = match kind {
            ModuleKind::Vertex => ShaderKind::Vertex,
            ModuleKind::Fragment => ShaderKind::Fragment,
            ModuleKind::Compute => ShaderKind::Compute,
        };

        graphics.device().translate_glsl_spirv(
            &source, &file_name, "main", kind,
            |target, _type, current, depth| {
                Ok(handle_include(current, _type, target, depth, assets, &snippets).unwrap())
            },
        ).map_err(ShaderCompilationError::TranslationError)?
    };
    log::debug!(
        "Took {:?} to translate '{}' to SPIRV",
        i.elapsed(),
        &file_name
    );
    Ok(artifacts)
}

// Compile SPIRV bytecode to an actual Vulkan module
fn compile_spirv(
    artifacts: &CompilationArtifact,
    graphics: &Graphics,
    file_name: &str
) -> vk::ShaderModule {
    let i = Instant::now();
    let raw = unsafe {
        let spirv = artifacts.as_binary();
        graphics.device().compile_shader_module(spirv)
    };
    log::debug!(
        "Took {:?} to compile '{}' from SPIRV",
        i.elapsed(),
        &file_name
    );
    raw
}

// Reflect the given compiled SPIRV data (baka)
fn reflect_spirv<M: ShaderModule>(
    artifacts: &CompilationArtifact
) -> Reflected<M> {
    unsafe {
        let raw = spirv_reflect::create_shader_module(artifacts.as_binary_u8()).unwrap();
        Reflected::<M>::from_raw_parts(raw)
    }
}

// This is a compiled shader module that we can use in multiple pipelines
pub struct Compiled<M: ShaderModule> {
    // Vulkan related data
    raw: vk::ShaderModule,
    kind: ModuleKind,
    constants: Constants,
    reflected: Reflected<M>,

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

    // Get out the reflected data from the SPIRV bytecode
    pub fn reflected(&self) -> &Reflected<M> {
        &self.reflected
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
            constants: &self.constants,
        }
    }
}

// A description of a compiled shader module that we can use within a pipeline
pub struct CompiledDescription<'a> {
    pub(crate) entry: &'static CStr,
    pub(crate) flags: vk::PipelineShaderStageCreateFlags,
    pub(crate) kind: ModuleKind,
    pub(crate) module: &'a vk::ShaderModule,
    pub(crate) constants: &'a Constants,
}
