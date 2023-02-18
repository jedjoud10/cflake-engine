use crate::{
    FunctionModule, GpuPodRelaxed, Graphics, ShaderCompilationError,
    ShaderModule, VertexModule, ShaderPreprocessorError, Reflected,
};
use ahash::AHashMap;
use assets::Assets;
use naga::{
    valid::{ModuleInfo, ValidationError},
    Module, ShaderStage, WithSpan,
};
use snailquote::unescape;
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

        // Compile GLSL to Naga then to Wgpu
        let time = std::time::Instant::now();
        let (raw, naga) = compile(stage, graphics, assets, &snippets, source)?;
        log::debug!("Compiled shader {} sucessfully! Took {} ms", file_name, time.elapsed().as_millis());

        Ok(Compiled {
            raw: Arc::new(raw),
            stage,
            file_name: file_name.into(),
            _phantom,
            graphics: graphics.clone(),
            naga: Arc::new(naga),
        })
    }
}

// Parses the GLSL shader into a Naga module, then passes it to Wgpu
fn compile(
    stage: ShaderStage,
    graphics: &Graphics,
    assets: &Assets,
    snippets: &Snippets,
    source: String,
) -> Result<(wgpu::ShaderModule, naga::Module), ShaderCompilationError> {
    // Pre-process the shader source to get expand of shader directives
    let source = preprocess(source, assets, snippets)
        .map_err(ShaderCompilationError::PreprocessorError)?;
    
    // [GLSL -> Naga] parsing options 
    let options = naga::front::glsl::Options {
        stage,
        defines: naga::FastHashMap::default(),
    };
    
    // Compile the GLSL shader source to a Naga module
    let mut parser = naga::front::glsl::Parser::default();
    let module = parser
        .parse(&options, &source)
        .map_err(ShaderCompilationError::ParserError)?;

    // Compile the Wgpu shader
    Ok((graphics.device().create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Naga(Cow::Owned(module.clone())),
        }
    ), module))
}

// Pre-process the GLSL shader source and include files / snippets
fn preprocess(
    source: String,
    assets: &Assets,
    snippets: &Snippets,
) -> Result<String, ShaderPreprocessorError> {
    // Parse a possible include line and fetch the target file / snippet
    fn convert_to_target(line: &str) -> Option<String> {
        let line = line.trim();
        let valid = line.starts_with("#include");
        let output = line.replace("#include", "").trim().to_string();
        valid.then_some(output)
    }

    // Check if an include directive resembles like an asset path instead of a snippet
    fn resembles_asset_path(path: &str) -> Option<bool> {
        // Check if we start with an angle bracket
        let mut characters = path.chars();
        let first_angle_bracket_valid = characters.next()? == '<';
        let second_angle_bracket_valid = characters.last()? == '>';
        
        // If we have the brackets, check if extension is valid?
        let extension_valid = if first_angle_bracket_valid && second_angle_bracket_valid {
            // Check if extension is "glsl"
            let path = &path.trim()[1..];
            let path = &path[..(path.len() - 1)];
            let pathbuf = PathBuf::try_from(path).ok()?;
            let extension = pathbuf.extension()?.to_str()?;
            extension == "glsl"
        } else {
            false
        };

        // Combine all tests
        Some(extension_valid && first_angle_bracket_valid && second_angle_bracket_valid)
    }

    // Load a function module and write it to the output line
    fn load_function_module(
        path: &str,
        assets: &Assets,
    ) -> Result<String, ShaderPreprocessorError> {
        let path = &path.trim()[1..];
        let path = &path[..(path.len() - 1)];

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
        let name = unescape(name).unwrap();
        let snippet = snippets
            .get(&name)
            .ok_or(ShaderPreprocessorError::SnippetNotDefined(name))?;
        Ok(snippet.clone())
    }

    // Recursive include function that will call iself
    fn include(
        source: String,
        assets: &Assets,
        snippets: &Snippets,
        depth: u32
    ) -> Result<String, ShaderPreprocessorError> {
        // If we're too deep, assume that the user caused a cyclic reference, and return an error
        if depth > 20 {
            return Err(ShaderPreprocessorError::IncludeCyclicReference);
        }

        let mut lines = source.lines().map(str::to_string).collect::<Vec<String>>();
        for line in lines.iter_mut() {
            let trimmed = line.trim();

            // Handle the include directive and replace the line
            if trimmed.starts_with("#include") {
                // Convert the line into "target"
                let target = convert_to_target(line)
                    .ok_or(ShaderPreprocessorError::InvalidIncludeDirective)?;

                // Either load it as an asset or a snippet
                let output = if resembles_asset_path(&target).unwrap_or_default() {
                    load_function_module(&target, assets)
                } else {
                    load_snippet(&target, snippets)
                }?;

                // Recusrive function calls itself
                let output = include(
                    output,
                    assets,
                    snippets,
                    depth + 1
                )?;
                
                *line = output;
            } 
        }
        Ok(lines.join("\n"))
    }
    
    // Call this once (it's recusrive so we chilling)
    include(
        source,
        assets,
        snippets,
        0
    )
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