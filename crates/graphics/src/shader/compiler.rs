use crate::{
    FunctionModule, GpuPodRelaxed, Graphics, ShaderCompilationError,
    ShaderModule, VertexModule, ShaderPreprocessorError,
};
use ahash::AHashMap;
use assets::Assets;
use naga::{
    valid::{ModuleInfo, ValidationError},
    Module, ShaderStage, WithSpan,
};
use std::{
    any::TypeId, borrow::Cow, ffi::CStr, marker::PhantomData,
    path::PathBuf, time::Instant, sync::Arc,
};

// Type alias for snippets and constants
type Snippets = AHashMap<String, String>;
type Constants = AHashMap<u32, Vec<u8>>;

// This is a compiler that will take was GLSL code, convert it to SPIRV,
// then to an appropriate Vulkan shader module.
// This compiler also allows us to define constants and snippets before compilation
pub struct Compiler<M: ShaderModule> {
    // Needed for shaderc and Vulkan pipeline module
    stage: ShaderStage,
    source: String,
    file_name: String,

    // Definitions
    snippets: Snippets,
    constants: Constants,

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
    */

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
        let module = parse_glsl(stage, graphics, assets, snippets, source)?;

        // Convert the Naga module to SPIRV bytecode
        let bytecode = compile_to_spirv(graphics, &module, constants)?;
        let naga = Arc::new(module);

        // Compile the SPIRV bytecode
        let raw = compile_module(graphics, bytecode);

        Ok(Compiled {
            raw: Arc::new(raw),
            stage,
            file_name: file_name.into(),
            _phantom,
            graphics: graphics.clone(),
            naga,
        })
    }
}

// Parse the GLSL code to the intermediate naga representation
// This will also include the necessary #include directives
fn parse_glsl(
    stage: ShaderStage,
    graphics: &Graphics,
    assets: &Assets,
    snippets: Snippets,
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

// Compile the Naga representation into SPIRV
fn compile_to_spirv(
    graphics: &Graphics,
    module: &Module,
    constants: Constants,
) -> Result<Vec<u32>, ShaderCompilationError> {
    // Validate the Naga shader first
    let mut validator = graphics.validator().lock();
    let info = validator
        .validate(module)
        .map_err(ShaderCompilationError::NagaValidationError)?;

    // Convert to SPIRV bytecode
    let options = naga::back::spv::Options::default();
    let bytecode =
        naga::back::spv::write_vec(module, &info, &options, None)
            .map_err(ShaderCompilationError::SpirvOutError)?;
    
    // TODO: Somehow implement specialization constants

    Ok(bytecode)
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

// Handle dealing with the include directive (that works with asset paths and snippets)
fn handle_include(
    current: &str,
    target: &str,
    depth: usize,
    assets: &Assets,
    snippets: &AHashMap<String, String>,
) -> Result<String, ShaderPreprocessorError> {
    // Check if an include directive resembles like an asset path instead of a snippet
    fn resembles_asset_path(path: &str) -> bool {
        let value: fn(&str) -> Option<bool> = |path: &str| {
            // Check if extension is "glsl"
            let pathbuf = PathBuf::try_from(path).ok()?;
            let extension = pathbuf.extension()?.to_str()?;
            let extension_valid = extension == "glsl";

            // Convert to words and make sure we start with #include
            let mut words = path.split_whitespace();
            let first_word_valid = words.next()? == "#include";
            let second = words.next()?;

            // Check if we start with an angle bracket
            let mut characters = second.chars();
            let first_angle_bracket_valid = characters.next()? == '<';
            let second_angle_bracket_valid = characters.last()? == '>';

            // Combine all tests
            Some(extension_valid && first_word_valid && first_angle_bracket_valid && second_angle_bracket_valid)
        };
        value(path).unwrap_or_default()
    }

    // Load a function module and write it to the output line
    fn load_function_module(
        path: &str,
        assets: &Assets,
    ) -> Result<String, ShaderPreprocessorError> {
        // Make sure the path is something we can load (.glsl file)
        let pathbuf = PathBuf::try_from(path).unwrap();

        // Load the path from the asset manager
        let path = pathbuf.as_os_str().to_str().unwrap();
        let content = assets
            .load::<FunctionModule>(path)
            .map(|x| x.source)
            .map_err(ShaderPreprocessorError::FileAssetError)?;
        Ok(content)
    }

    // Load a snippet from the snippets and write it to the output line
    fn load_snippet(
        name: &str,
        snippets: &AHashMap<String, String>,
    ) -> Result<String, ShaderPreprocessorError> {
        let snippet = snippets
            .get(name)
            .ok_or(ShaderPreprocessorError::SnippetNotDefined(name.to_string()))?;
        Ok(snippet.clone())
    }

    // Either load it as an asset or a snippet
    if resembles_asset_path(&target) {
        load_function_module(target, assets)
    } else {
        load_snippet(target, snippets)
    }
}

// This is a compiled shader module that we can use in multiple pipelines
// We can clone this shader module since we can share 
pub struct Compiled<M: ShaderModule> {
    // Wgpu related data
    raw: Arc<wgpu::ShaderModule>,
    naga: Arc<naga::Module>,
    stage: ShaderStage,

    // Helpers
    file_name: Arc<str>,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Clone for Compiled<M> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            naga: self.naga.clone(),
            stage: self.stage.clone(),
            file_name: self.file_name.clone(),
            _phantom: self._phantom.clone(),
            graphics: self.graphics.clone(),
        }
    }
}

impl<M: ShaderModule> Compiled<M> {
    // Get the raw wgpu hidden module
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.raw
    }

    // Get the shader module stage for this compiled shader
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    // Get the shader module file name for this module
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    // Get the internally stored Naga representation of the shader
    pub fn naga(&self) -> &naga::Module {
        &self.naga
    }

    // Get the entry point for the compiled shader
    pub fn entry_point(&self) -> Option<&str> {
        self.naga.entry_points.iter().next().map(|n| n.name.as_str())
    }
}