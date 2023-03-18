use crate::{
    BindResourceType, Buffer, ViewDimension, FunctionModule, GpuPod,
    GpuPodInfo, Graphics, ModuleKind, ModuleVisibility,
    PushConstantLayout, ReflectedShader, ShaderCompilationError,
    ShaderError, ShaderModule, ShaderReflectionError, Texel,
    TexelInfo, Texture, VertexModule, Region, Extent,
};
use ahash::AHashMap;
use assets::Assets;
use itertools::Itertools;
use snailquote::unescape;
use std::{
    any::TypeId,
    borrow::Cow,
    ffi::CStr,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use thiserror::Error;

use super::create_pipeline_layout;

// Type alias for snippets and resources
pub(super) type Snippets = AHashMap<String, String>;
pub(super) type ResourceBindingTypes = AHashMap<String, BindResourceType>;
pub(super) type MaybePushConstantLayout = Option<PushConstantLayout>;

// This is a compiler that will take was GLSL code, convert it to Naga, then to WGPU
// This compiler also allows us to define constants and snippets before compilation
// This compiler will be used within the Shader and ComputeShader to compile the modules in batch
pub struct Compiler<'a> {
    assets: &'a Assets,
    graphics: &'a Graphics,
    snippets: Snippets,
    resource_types: ResourceBindingTypes,
    maybe_push_constant_layout: MaybePushConstantLayout,
}

impl<'a> Compiler<'a> {
    // Create a new default compiler with the asset loader
    pub fn new(assets: &'a Assets, graphics: &'a Graphics) -> Self {
        Self {
            assets,
            graphics,
            snippets: Default::default(),
            resource_types: Default::default(),
            maybe_push_constant_layout: Default::default(),
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

    // Convert the given GLSL code to SPIRV code, then compile said SPIRV code
    // This uses the defined resoures defined in this compiler
    pub(crate) fn compile<M: ShaderModule>(
        &self,
        module: M,
    ) -> Result<Compiled<M>, ShaderError> {
        // Decompose the module into file name and source
        let (name, source) = module.into_raw_parts();

        // Compile GLSL to Naga then to Wgpu
        let time = std::time::Instant::now();
        let (raw, naga) = compile(
            &M::kind(),
            &self.graphics,
            &self.assets,
            &self.snippets,
            source,
            &name,
        )
        .map_err(ShaderError::Compilation)?;
        log::debug!(
            "Compiled shader {name} sucessfully! Took {}ms",
            time.elapsed().as_millis()
        );

        Ok(Compiled {
            raw: Arc::new(raw),
            name: name.into(),
            _phantom: PhantomData,
            graphics: self.graphics.clone(),
            naga: Arc::new(naga),
        })
    }

    // Convert the given shader modules
    pub(crate) fn create_pipeline_layout(
        &self,
        names: &[&str],
        modules: &[&naga::Module],
        visibility: &[ModuleVisibility],
    ) -> Result<
        (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>),
        ShaderError,
    > {
        create_pipeline_layout(
            self.graphics,
            names,
            modules,
            visibility,
            &self.resource_types,
            &self.maybe_push_constant_layout,
        )
        .map_err(ShaderError::Reflection)
    }
}

impl<'a> Compiler<'a> {
    // Inserts a bind resource type into the compiler resource definitions
    // Logs out a debug message if one of the resources gets overwritten
    pub fn use_resource_type(
        &mut self,
        name: impl ToString,
        resource: BindResourceType,
    ) {
        let name = name.to_string();
        match self.resource_types.entry(name.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                log::info!("Binding resource '{name}' was replaced");
                occupied.insert(resource);
            },
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(resource);
            },
        }
    }

    // Define a uniform buffer type's inner struct type
    pub fn use_uniform_buffer<T: GpuPod>(
        &mut self,
        name: impl ToString,
    ) {
        let size = T::size();
        self.use_resource_type(
            name,
            BindResourceType::UniformBuffer { size }
        );
    }

    // Define a storage buffer type's inner struct type
    pub fn use_storage_buffer<T: GpuPod>(
        &mut self,
        name: impl ToString,
        read: bool,
        write: bool,
    ) {
        let size = T::size();
        self.use_resource_type(
            name,
            BindResourceType::StorageBuffer {
                size,
                read,
                write
            }
        );
    }

    // Define a uniform sampled texture's type and texel
    pub fn use_sampled_texture<T: Texture>(&mut self, name: impl ToString) {
        let sampler_name = format!("{}_sampler", name.to_string());
        self.use_sampler::<T::T>(sampler_name);
        
        let dimensionality = <<T::Region as Region>::E as Extent>::view_dimension();
        let info = <T::T as Texel>::info();
        let format = info.format();

        self.resource_types
            .insert(name.to_string(), BindResourceType::SampledTexture {
                format,
                sample_type: super::map_texture_sample_type(&self.graphics, info),
                sampler_binding: super::map_sampler_binding_type(&self.graphics, info),
                view_dimension: dimensionality,
            });
    }

    // Define a uniform sampler's type and texel
    pub fn use_sampler<T: Texel>(&mut self, name: impl ToString) {
        let info = <T as Texel>::info();
        let format = info.format();

        self.resource_types
            .insert(name.to_string(), BindResourceType::Sampler {
                format: format,
                sampler_binding: super::map_sampler_binding_type(&self.graphics, info)
            });
    }

    // Define a push constant range to be pushed
    pub fn use_push_constant_layout(
        &mut self,
        layout: PushConstantLayout,
    ) {
        self.maybe_push_constant_layout = Some(layout);
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

    // Create a callback responsible for includes
    options.set_include_callback(|target, _type, current, depth| {
        include(current, _type, target, depth, assets, &snippets)
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

                ShaderCompilationError::ShaderC
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

    // TODO: Figure out if we should keep naga here or simply not use it
    // Sole reason we have it rn is so we can get the binding index and set index automatically

    // Compile the Wgpu shader
    let wgpu = graphics.device().create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Naga(Cow::Owned(
                module.clone(),
            )),
        },
    );

    // Return the compiled wgpu module and the naga mdule
    Ok((wgpu, module))
}

// Load a function module and convert it to a ResolvedInclude
fn load_function_module(
    path: &str,
    assets: &Assets,
) -> Result<shaderc::ResolvedInclude, String> {
    // Make sure the path is something we can load (.glsl file)
    let pathbuf = PathBuf::try_from(path).unwrap();

    // Load the path from the asset manager
    let path = pathbuf.as_os_str().to_str().unwrap();
    let content = assets
        .load::<FunctionModule>(path)
        .map(|x| x.source)
        .map_err(|err| format!("File include error: {err:?}"))?;
    Ok(shaderc::ResolvedInclude {
        resolved_name: path.to_string(),
        content,
    })
}

// Load a snippet from the snippets and convert it to a ResolvedInclude
fn load_snippet(
    name: &str,
    snippets: &AHashMap<String, String>,
) -> Result<shaderc::ResolvedInclude, String> {
    let snippet = snippets.get(name).ok_or(format!(
        "Snippet {} was not defined",
        name.to_string()
    ))?;
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
    if depth > 40 {
        return Err(format!("Include cyclic reference detected"));
    }

    // Check if the user wants to load a snippet or asset
    // If it's a snippet, then the name of the snippet should be surrounded with ""
    // If it's an asset, then the name of the file should be surrounded with <>
    let resembles = matches!(_type, shaderc::IncludeType::Standard);

    // Either load it as an asset or a snippet
    let output = if resembles {
        log::debug!("Loading shader function module '{target}'");
        load_function_module(&target, assets)
    } else {
        log::debug!("Loading shader source snippet '{target}'");
        load_snippet(&target, snippets)
    };

    // Convert the error to a string instead
    output
}

// This is a compiled shader module that we can use in multiple pipelines
// We can clone this shader module since we should be able to share them
pub struct Compiled<M: ShaderModule> {
    // Wgpu related data
    raw: Arc<wgpu::ShaderModule>,

    // FIXME: Possibly remove this shit?
    naga: Arc<naga::Module>,

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
            name: self.name.clone(),
            _phantom: self._phantom.clone(),
            graphics: self.graphics.clone(),
            naga: self.naga.clone(),
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

    // Get the underlying raw Naga module
    pub fn naga(&self) -> &naga::Module {
        &self.naga
    }

    // Get the shader module name for this module
    pub fn name(&self) -> &str {
        &self.name
    }
}
