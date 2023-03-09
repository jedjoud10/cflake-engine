use crate::{
    FunctionModule, GpuPodRelaxed, Graphics, ModuleKind,
    ReflectedModule, ShaderCompilationError, ShaderModule,
    VertexModule, BindResourceType, Texture, Texel, TexelInfo, ShaderIncludeError,
};
use ahash::AHashMap;
use assets::Assets;
use itertools::Itertools;
use naga::{
    valid::{ModuleInfo, ValidationError},
    Module, ShaderStage, WithSpan,
};
use snailquote::unescape;
use thiserror::Error;
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
    assets: &'a Assets,
    snippets: Snippets,
    texture_formats: TextureFormats,
}

impl<'a> Compiler<'a> {
    // Create a new default compiler with the asset loader 
    pub fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            snippets: Default::default(),
            texture_formats: Default::default(),
        }
    }

    // Include a snippet directive that will replace #includes surrounded by ""
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
        &self,
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
            &self.assets,
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
        let reflected = super::reflect_module::<M>(&graphics, &naga, &self.texture_formats);
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
    assets: &Assets,
    snippets: &Snippets,
    source: String,
    file: &str,
) -> Result<(wgpu::ShaderModule, naga::Module), ShaderCompilationError> {
    // Custom ShaderC compiler options
    let mut options = shaderc::CompileOptions::new().unwrap();
    // FIXME: OwO what's this??
    //options.set_auto_combined_image_sampler(auto_combine);

    // Create a callback responsible for includes
    options.set_include_callback(|target, _type, current, depth| {
        include(
            current, _type, target, depth, assets,
            &snippets,
        )
    });

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
            Some(&options),
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

                // Simple message containing the file that contains the error
                log::error!("Failed compilation of shader '{file}'");

                // Print a preview of the file with counted lines
                log::error!("Source code: \n\n{source}\n\n");                

                // Print the error message
                for line in value.lines() {
                    log::error!("{}", line);
                }

                ShaderCompilationError
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
    let module = naga::front::spv::parse_u8_slice(
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

// Load a function module and convert it to a ResolvedInclude
fn load_function_module(
    path: &str,
    assets: &Assets,
) -> Result<shaderc::ResolvedInclude, ShaderIncludeError> {
    // Make sure the path is something we can load (.glsl file)
    let pathbuf = PathBuf::try_from(path).unwrap();

    // Load the path from the asset manager
    let path = pathbuf.as_os_str().to_str().unwrap();
    let content = assets
        .load::<FunctionModule>(path)
        .map(|x| x.source)
        .map_err(ShaderIncludeError::FileAssetError)?;
    Ok(shaderc::ResolvedInclude {
        resolved_name: path.to_string(),
        content,
    })
}

// Load a snippet from the snippets and convert it to a ResolvedInclude
fn load_snippet(
    name: &str,
    snippets: &AHashMap<String, String>,
) -> Result<shaderc::ResolvedInclude, ShaderIncludeError> {
    let snippet = snippets.get(name).ok_or(
        ShaderIncludeError::SnippetNotDefined(name.to_string()),
    )?;
    Ok(shaderc::ResolvedInclude {
        resolved_name: name.to_string(),
        content: snippet.clone(),
    })
}

// Include callback that will be passed to the ShaderC compiler
fn include(
    _current: &str,
    _type: shaderc::IncludeType,
    target: &str,
    depth: usize,
    assets: &Assets,
    snippets: &Snippets,
) -> Result<shaderc::ResolvedInclude, String> {
    // If we're too deep, assume that the user caused a cyclic reference, and return an error
    if depth > 20 {
        return Err(
            format!("{:?}", ShaderIncludeError::IncludeCyclicReference)
        );
    }

    // Check if the user wants to load a snippet or asset
    // If it's a snippet, then the name of the snippet should be surrounded with ""
    // If it's an asset, then the name of the file should be surrounded with <>
    let resembles = matches!(_type, shaderc::IncludeType::Standard);

    // Either load it as an asset or a snippet
    let output = if resembles {
        log::debug!(
            "Loading shader function module '{target}'"
        );
        load_function_module(&target, assets)
    } else {
        log::debug!(
            "Loading shader source snippet '{target}'"
        );
        load_snippet(&target, snippets)
    };

    if output.is_err() {
        log::warn!("Hang yourself");
    }

    // Convert the error to a string instead
    output.map_err(|err| format!("{err:?}"))
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
