use crate::{
    FunctionModule, GpuPodRelaxed, Graphics, ShaderCompilationError,
    ShaderIncludeError, ShaderModule,
};
use ahash::AHashMap;
use assets::Assets;
use naga::{
    valid::{ModuleInfo, ValidationError},
    Module, ShaderStage, WithSpan,
};
use std::{
    any::TypeId, borrow::Cow, ffi::CStr, marker::PhantomData,
    path::PathBuf, time::Instant,
};

// This is a compiler that will take was GLSL code, convert it to SPIRV,
// then to an appropriate Vulkan shader module.
// This compiler also allows us to define constants and snippets before compilation
pub struct Compiler<M: ShaderModule> {
    // Needed for shaderc and Vulkan pipeline module
    stage: ShaderStage,
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
        let stage = module.stage();
        let (file_name, source) = module.into_raw_parts();
        log::debug!("Created a new compiler for {}", file_name);

        Self {
            stage,
            source,
            file_name,
            _phantom: PhantomData,
            snippets: Default::default(),
            constants: Default::default(),
        }
    }

    /*
    TODO: Re-implement this
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
    */

    // Convert the GLSL code to SPIRV code, then compile said SPIRV code
    pub fn compile(
        self,
        assets: &Assets,
        graphics: &Graphics,
    ) -> Result<Compiled<M>, ShaderCompilationError> {
        let Compiler {
            stage,
            source,
            file_name,
            snippets,
            constants,
            _phantom,
        } = self;

        // Convert GLSL to the Naga module (and validate it)
        let module = parse_glsl(stage, graphics, source)?;
        let info = validate(&graphics, &module)?;

        // Convert the Naga module to SPIRV bytecode
        let bytecode = compile_to_spirv(module, info)?;

        // Compile the SPIRV bytecode
        let raw = compile_module(graphics, bytecode);

        Ok(Compiled {
            raw,
            stage,
            file_name,
            _phantom,
            graphics: graphics.clone(),
        })
    }
}

// Compile the SPIRV shader
fn compile_module(
    graphics: &Graphics,
    bytecode: Vec<u32>,
) -> wgpu::ShaderModule {
    let raw = unsafe {
        graphics.device().create_shader_module_spirv(
            &wgpu::ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Borrowed(&bytecode),
            },
        )
    };
    raw
}

// Parse the GLSL code to the intermediate naga representation
fn parse_glsl(
    stage: ShaderStage,
    graphics: &Graphics,
    source: String,
) -> Result<Module, ShaderCompilationError> {
    let options = naga::front::glsl::Options {
        stage,
        defines: naga::FastHashMap::default(),
    };
    let mut parser = graphics.parser().lock();
    let module = parser
        .parse(&options, &source)
        .map_err(ShaderCompilationError::ParserError)?;
    Ok(module)
}

// Validate a naga Module
fn validate(
    graphics: &Graphics,
    module: &Module,
) -> Result<ModuleInfo, ShaderCompilationError> {
    let mut validator = graphics.validator().lock();
    validator
        .validate(module)
        .map_err(ShaderCompilationError::NagaValidationError)
}

// Compile the Naga representation into SPIRV
fn compile_to_spirv(
    module: Module,
    info: ModuleInfo,
) -> Result<Vec<u32>, ShaderCompilationError> {
    let options = naga::back::spv::Options::default();
    let bytecode =
        naga::back::spv::write_vec(&module, &info, &options, None)
            .map_err(ShaderCompilationError::SpirvOutError)?;
    Ok(bytecode)
}

// Data that must be stored within the compiled shader
// that indicates how constants are defined in the specialization info
pub struct Constants {}

/*
// Calculate the specialization info based on a hashmap of constants
fn create_constants_wrapper(
    constants: AHashMap<u32, Vec<u8>>,
) -> Constants {
    let merged = constants.iter().collect::<Vec<_>>();
    let data = merged
        .iter()
        .flat_map(|(_, data)| data.iter().cloned())
        .collect::<Vec<_>>();
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

    Constants { raw, data, entries }
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
        let resolved_name =
            pathbuf.clone().into_os_string().into_string().unwrap();
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
        let snippet = snippets
            .get(name)
            .ok_or(ShaderIncludeError::SnippetNotDefined)?;
        Ok(ResolvedInclude {
            resolved_name: name.to_string(),
            content: snippet.clone(),
        })
    }

    // Relative paths not supported yet
    assert!(
        matches!(_type, IncludeType::Standard),
        "Not supported yet"
    );

    // Either load it as an asset or a snippet
    if resembles_asset_path(&target) {
        load_function_module(target, assets)
    } else {
        load_snippet(target, snippets)
    }
}
*/

// This is a compiled shader module that we can use in multiple pipelines
pub struct Compiled<M: ShaderModule> {
    // Wgpu related data
    raw: wgpu::ShaderModule,
    stage: ShaderStage,

    // Helpers
    file_name: String,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Compiled<M> {
    // Get the shader module stage for this compiled shader
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    // Get the shader module file name for this module
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}
