use crate::{
    FunctionModule, GpuPodRelaxed, Graphics, ModuleKind,
    ReflectedModule, ShaderCompilationError, ShaderModule,
    ShaderPreprocessorError, VertexModule, BindResourceType, Texture, Texel, TexelInfo,
};
use ahash::AHashMap;
use assets::Assets;
use itertools::Itertools;
use naga::{
    valid::{ModuleInfo, ValidationError},
    Module, ShaderStage, WithSpan,
};
use snailquote::unescape;
use std::{
    any::TypeId, borrow::Cow, ffi::CStr, marker::PhantomData,
    path::PathBuf, sync::Arc, time::Instant,
};

// Type alias for snippets and resources
pub(super) type Snippets = AHashMap<String, String>;
pub(super) type TextureFormats = AHashMap<String, TexelInfo>; 

// This is a compiler that will take was GLSL code, convert it to Naga, then to WGPU
// This compiler also allows us to define constants and snippets before compilation
// This compiler will be used within the Shader and ComputeShader to compile the modules in batch
pub struct Compiler<'a> {
    assets: &'a mut Assets,
    snippets: Snippets,
    texture_formats: TextureFormats,
}

impl<'a> Compiler<'a> {
    // Create a new default compiler with the asset loader 
    pub fn new(assets: &'a mut Assets) -> Self {
        Self {
            assets,
            snippets: Default::default(),
            texture_formats: Default::default(),
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
    pub fn use_snippet(
        &mut self,
        name: impl ToString,
        value: impl ToString,
    ) {
        let name = name.to_string();
        self.snippets.insert(name, value.to_string());
    }

    // Define a uniform buffer type's inner struct type
    // TODO: Figure out a way to make this useful
    pub fn use_ubo<T: GpuPodRelaxed>(
        &mut self,
        name: impl ToString,
    ) {
    }

    // Define a "fill" uniform buffer whose layout is defined at runtime
    // TODO: Figure out a way to make this useful
    pub fn use_fill_ubo(
        &mut self,
        name: impl ToString,
    ) {
    }

    // Define a uniform texture's type and texel
    pub fn use_texture<T: Texture>(
        &mut self,
        name: impl ToString,
    ) {
        let name = name.to_string();
        let sampler = format!("{name}_sampler");
        self.texture_formats.insert(name, <T::T as Texel>::info());
        self.set_sampler::<T>(sampler);
    }

    // Define a uniform sampler's type and texel
    // This is called automatically if the sampler is bound to the texture
    fn set_sampler<T: Texture>(
        &mut self,
        name: impl ToString,
    ) {
        let name = name.to_string();
        self.texture_formats.insert(name, <T::T as Texel>::info());
    }

    // Convert the given GLSL code to SPIRV code, then compile said SPIRV code
    // This uses the defined resoures defined in this compiler
    pub(crate) fn compile<M: ShaderModule>(
        &mut self,
        module: M,
        graphics: &Graphics,
    ) -> Result<Compiled<M>, ShaderCompilationError> {
        // Decompose the module into file name and source
        let (name, source) = module.into_raw_parts();
        
        // Compile GLSL to Naga then to Wgpu
        let time = std::time::Instant::now();
        let (raw, naga) = compile(
            &M::kind(),
            graphics,
            &mut self.assets,
            &self.snippets,
            source,
            &name,
        )?;
        log::debug!(
            "Compiled shader {name} sucessfully! Took {}ms",
            time.elapsed().as_millis()
        );

        // Reflect the module with the given bind layout
        let time = std::time::Instant::now();
        let reflected = super::reflect_module::<M>(&naga, &self.texture_formats);
        log::debug!(
            "Reflected shader {name} sucessfully! Took {}ms",
            time.elapsed().as_millis()
        );

        Ok(Compiled {
            raw: Arc::new(raw),
            naga: Arc::new(naga),
            reflected: Arc::new(reflected),
            name: name.into(),
            _phantom: PhantomData,
            graphics: graphics.clone(),
        })
    }
}

// Parses the GLSL shader into a Naga module, then passes it to Wgpu
// If the underlying shader module is cached, it will use that
fn compile(
    kind: &ModuleKind,
    graphics: &Graphics,
    assets: &mut Assets,
    snippets: &Snippets,
    source: String,
    file: &str,
) -> Result<(wgpu::ShaderModule, naga::Module), ShaderCompilationError> {
    // Pre-process the shader source to get expand of shader directives
    let source = preprocess(source, assets, snippets)
        .map_err(ShaderCompilationError::PreprocessorError)?;

    // Pass the source by ShaderC first cause Naga's errors suck ass
    let artifact = graphics
        .0
        .shaderc
        .compile_into_spirv(
            &source,
            match kind {
                ModuleKind::Vertex => shaderc::ShaderKind::Vertex,
                ModuleKind::Fragment => shaderc::ShaderKind::Fragment,
                ModuleKind::Compute => shaderc::ShaderKind::Compute,
            },
            file,
            "main",

            // TODO: Use this shit
            None,
        )
        .map_err(|error| match error {
            // ShaderC compilation error, so print out the message to the error log
            shaderc::Error::CompilationError(_, value) => {
                // Get the source code for this stage, and identify each line with it's line out
                let source = source
                    .lines()
                    .enumerate()
                    .map(|(count, line)| {
                        format!("({}): {}", count + 1, line)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                // Print the error message
                log::error!(
                    "Failed compilation of shader {file}:\n\n{}\n\n",
                    source
                );

                for line in value.lines() {
                    log::error!("{}", line);
                }

                ShaderCompilationError::ValidationError
            }
            _ => todo!(),
        })?;

    // [SPIRV -> Naga] parsing options
    let options = naga::front::spv::Options {
        adjust_coordinate_space: false,
        strict_capabilities: false,
        block_ctx_dump_prefix: None,
    };

    // Compile the SPIRV to a Naga module
    let mut module = naga::front::spv::parse_u8_slice(
        artifact.as_binary_u8(),
        &options,
    )
    .unwrap();

    // Compile the Wgpu shader
    Ok((
        graphics.device().create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Naga(Cow::Owned(
                    module.clone(),
                )),
            },
        ),
        module,
    ))
}

// Pre-process the GLSL shader source and include files / snippets
fn preprocess(
    source: String,
    assets: &mut Assets,
    snippets: &Snippets,
) -> Result<String, ShaderPreprocessorError> {
    // Cleanse shader input by removing comments and commented code
    // TODO: Implement this pleasee https://blog.ostermiller.org/finding-comments-in-source-code-using-regular-expressions/
    fn cleanse(source: String) -> String {
        source
    }

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
        let extension_valid = if first_angle_bracket_valid
            && second_angle_bracket_valid
        {
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
        Some(
            extension_valid
                && first_angle_bracket_valid
                && second_angle_bracket_valid,
        )
    }

    // Load a function module and write it to the output line
    fn load_function_module(
        path: &str,
        assets: &mut Assets,
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
        let snippet = snippets.get(&name).ok_or(
            ShaderPreprocessorError::SnippetNotDefined(name),
        )?;
        Ok(snippet.clone())
    }

    // Recursive include function that will call iself
    fn include(
        source: String,
        assets: &mut Assets,
        snippets: &Snippets,
        depth: u32,
    ) -> Result<String, ShaderPreprocessorError> {
        // If we're too deep, assume that the user caused a cyclic reference, and return an error
        if depth > 20 {
            return Err(
                ShaderPreprocessorError::IncludeCyclicReference,
            );
        }

        let mut lines = source
            .lines()
            .map(str::to_string)
            .collect::<Vec<String>>();
        for line in lines.iter_mut() {
            let trimmed = line.trim();

            // Handle the include directive and replace the line
            if trimmed.starts_with("#include") {
                // Convert the line into "target"
                let target = convert_to_target(line).ok_or(
                    ShaderPreprocessorError::InvalidIncludeDirective,
                )?;

                // Either load it as an asset or a snippet
                let output = if resembles_asset_path(&target)
                    .unwrap_or_default()
                {
                    log::debug!(
                        "Loading shader function module '{target}'"
                    );
                    load_function_module(&target, assets)
                } else {
                    log::debug!(
                        "Loading shader source snippet '{target}'"
                    );
                    load_snippet(&target, snippets)
                }?;

                // Recusrive function calls itself
                let output =
                    include(output, assets, snippets, depth + 1)?;

                *line = output;
            }
        }
        Ok(lines.join("\n"))
    }

    // Cleanse shader input
    let source = cleanse(source);

    // Call this once (it's recusrive so we chilling)
    include(source, assets, snippets, 0)
}

// This is a compiled shader module that we can use in multiple pipelines
// We can clone this shader module since we should be able to share them
pub struct Compiled<M: ShaderModule> {
    // Wgpu related data
    raw: Arc<wgpu::ShaderModule>,
    naga: Arc<naga::Module>,
    reflected: Arc<ReflectedModule>,

    // Helpers
    name: Arc<str>,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Clone for Compiled<M> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            naga: self.naga.clone(),
            reflected: self.reflected.clone(),
            name: self.name.clone(),
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

    // Get the reflected shader representation
    pub fn reflected(&self) -> &ReflectedModule {
        &self.reflected
    }

    // Get the shader module name for this module
    pub fn name(&self) -> &str {
        &self.name
    }

    // Get the internally stored Naga representation of the shader
    pub fn naga(&self) -> &naga::Module {
        &self.naga
    }
}
