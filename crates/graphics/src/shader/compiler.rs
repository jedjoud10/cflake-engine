use crate::{
    BindResourceType, Buffer, Extent, FunctionModule, GpuPod,
    GpuPodInfo, Graphics, ModuleKind, ModuleVisibility,
    PushConstantLayout, ReflectedShader, Region,
    ShaderCompilationError, ShaderError, ShaderModule,
    ShaderReflectionError, SpecConstant, Texel, TexelInfo, Texture,
    VertexModule, ViewDimension,
};
use ahash::{AHashMap, AHashSet};
use assets::Assets;
use itertools::Itertools;
use parking_lot::Mutex;
use snailquote::unescape;
use std::{
    any::TypeId,
    borrow::Cow,
    collections::BTreeMap,
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
pub(crate) type Snippets = BTreeMap<String, String>;
pub(crate) type ResourceBindingTypes =
    AHashMap<String, BindResourceType>;
pub(crate) type MaybePushConstantLayout = Option<PushConstantLayout>;
pub(crate) type Included = Arc<Mutex<AHashSet<String>>>;
pub(crate) type Constants = AHashMap<u32, spirq::ConstantValue>;

// This is a compiler that will take GLSL code and create a WGPU module
// This compiler also allows us to define constants and snippets before compilation
// This compiler will be used within the Shader and ComputeShader to compile the modules in batch
pub struct Compiler<'a> {
    pub(crate) assets: &'a Assets,
    pub(crate) graphics: &'a Graphics,
    pub(crate) snippets: Snippets,
    pub(crate) constants: Constants,
    pub(crate) resource_types: ResourceBindingTypes,
    pub(crate) maybe_push_constant_layout: MaybePushConstantLayout,
    optimization: shaderc::OptimizationLevel,
}

impl<'a> Compiler<'a> {
    // Create a new default compiler with the asset loader
    pub fn new(assets: &'a Assets, graphics: &'a Graphics) -> Self {
        Self {
            assets,
            graphics,
            snippets: Default::default(),
            constants: Default::default(),
            resource_types: Default::default(),
            maybe_push_constant_layout: Default::default(),
            optimization: shaderc::OptimizationLevel::Zero,
        }
    }

    // Set the value of a specilization constant within the shader
    // TODO: Find a library that will specialize the constants at runtime ffs
    pub fn use_specialization_constant(
        &mut self,
        id: u32,
        value: impl SpecConstant,
    ) {
        todo!()
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

    // Set the optimization level used by the ShaderC compiler
    pub fn use_optimization_level(
        &mut self,
        level: shaderc::OptimizationLevel,
    ) {
        self.optimization = level;
    }

    // Convert the given GLSL code to SPIRV code, then compile said SPIRV code
    // This uses the defined resoures defined in this compiler
    pub(crate) fn compile<M: ShaderModule>(
        &self,
        module: M,
    ) -> Result<Compiled<M>, ShaderError> {
        // Decompose the module into file name and source
        let (name, source) = module.into_raw_parts();

        // Compile GLSL to SPIRV then to Wgpu
        let time = std::time::Instant::now();
        let (raw, reflected) = compile(
            &M::kind(),
            &self.graphics,
            &self.assets,
            &self.snippets,
            &self.constants,
            self.optimization,
            source,
            &name,
        )
        .map_err(ShaderError::Compilation)?;
        log::debug!(
            "Compiled shader {name} sucessfully! Took {}ms",
            time.elapsed().as_millis()
        );

        Ok(Compiled {
            raw,
            name: name.into(),
            _phantom: PhantomData,
            graphics: self.graphics.clone(),
            reflected,
        })
    }

    // Convert the given shader modules
    pub(crate) fn create_pipeline_layout(
        &self,
        names: &[&str],
        modules: &[&spirq::EntryPoint],
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
            std::collections::hash_map::Entry::Occupied(
                mut occupied,
            ) => {
                log::debug!("Binding resource '{name}' was replaced");
                occupied.insert(resource);
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(resource);
            }
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
            BindResourceType::UniformBuffer { size },
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
            BindResourceType::StorageBuffer { size, read, write },
        );
    }

    // Define a uniform sampled texture's type and texel
    pub fn use_sampled_texture<T: Texture>(
        &mut self,
        name: impl ToString,
    ) {
        let sampler_name = format!("{}_sampler", name.to_string());
        self.use_sampler::<T::T>(sampler_name);

        let dimensionality =
            <<T::Region as Region>::E as Extent>::view_dimension();
        let info = <T::T as Texel>::info();
        let format = info.format();

        self.resource_types.insert(
            name.to_string(),
            BindResourceType::SampledTexture {
                format,
                sample_type: super::map_texture_sample_type(
                    &self.graphics,
                    info,
                ),
                sampler_binding: super::map_sampler_binding_type(
                    &self.graphics,
                    info,
                ),
                view_dimension: dimensionality,
            },
        );
    }

    // Define a uniform sampler's type and texel
    pub fn use_sampler<T: Texel>(&mut self, name: impl ToString) {
        let info = <T as Texel>::info();
        let format = info.format();

        self.resource_types.insert(
            name.to_string(),
            BindResourceType::Sampler {
                format: format,
                sampler_binding: super::map_sampler_binding_type(
                    &self.graphics,
                    info,
                ),
            },
        );
    }

    // Define a storage texture that we can read / write to
    pub fn use_storage_texture<T: Texture>(
        &mut self,
        name: impl ToString,
        read: bool,
        write: bool,
    ) {
        let dimensionality =
            <<T::Region as Region>::E as Extent>::view_dimension();
        let info = <T::T as Texel>::info();
        let format = info.format();

        self.resource_types.insert(
            name.to_string(),
            BindResourceType::StorageTexture {
                access: match (read, write) {
                    (true, true) => {
                        wgpu::StorageTextureAccess::ReadWrite
                    }
                    (true, false) => {
                        wgpu::StorageTextureAccess::ReadOnly
                    }
                    (false, true) => {
                        wgpu::StorageTextureAccess::WriteOnly
                    }
                    _ => todo!(),
                },
                format,
                sample_type: super::map_texture_sample_type(
                    &self.graphics,
                    info,
                ),
                view_dimension: dimensionality,
            },
        );
    }

    // Define a push constant range to be pushed
    pub fn use_push_constant_layout(
        &mut self,
        layout: PushConstantLayout,
    ) {
        self.maybe_push_constant_layout = Some(layout);
    }
}

// Parses the GLSL shader into a SPIRV module, then passes it to Wgpu
// If the underlying shader module is cached, it will use that
fn compile(
    kind: &ModuleKind,
    graphics: &Graphics,
    assets: &Assets,
    snippets: &Snippets,
    constants: &Constants,
    optimization: shaderc::OptimizationLevel,
    source: String,
    file: &str,
) -> Result<
    (Arc<wgpu::ShaderModule>, Arc<spirq::EntryPoint>),
    ShaderCompilationError,
> {
    // If the shader cache already contains the compiled shader, simply reuse it
    // TODO: Holy fuck please optimize this
    // TODO: Also change cache to LruCache or smthing like that
    if let Some(value) = graphics
        .0
        .cached
        .shaders
        .get(&(snippets.clone(), file.to_string()))
    {
        let (raw, reflected) = value.value();
        log::debug!(
            "Found shader module in cache for {file}, using it..."
        );
        return Ok((raw.clone(), reflected.clone()));
    } else {
        log::warn!("Did not find cached shader module for {file}");
    }

    // Custom ShaderC compiler options
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_generate_debug_info();
    options.set_optimization_level(optimization);
    options.set_invert_y(false);

    // Keeps track of what files/snippets where included
    // TODO: File bug report cause I'm pretty sure it's supposed to *not* add duplicate includes
    let included = Included::default();

    // Create a callback responsible for includes
    options.set_include_callback(
        move |target, _type, current, depth| {
            include(
                current, _type, target, depth, assets, &snippets,
                &included,
            )
        },
    );

    // Compile using ShaderC (my love)
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

    // Print out possible warning messages during shader compilation
    if !artifact.get_warning_messages().is_empty() {
        log::warn!(
            "ShaderC warning: {}",
            artifact.get_warning_messages()
        );
    }

    // Setup basic config spirq option
    let reflect = spirq::ReflectConfig::new()
        .spv(artifact.as_binary())
        .combine_img_samplers(false)
        .ref_all_rscs(true)
        .gen_unique_names(false)
        .reflect()
        .unwrap()
        .pop()
        .unwrap();

    // Compile the Wgpu shader (raw spirv passthrough)
    let wgpu = unsafe {
        graphics.device().create_shader_module_spirv(
            &wgpu::ShaderModuleDescriptorSpirV {
                label: Some(&format!("shader-module-{file}")),
                source: wgpu::util::make_spirv_raw(
                    artifact.as_binary_u8(),
                ),
            },
        )
    };

    // Cache the result first
    let raw = Arc::new(wgpu);
    let reflected = Arc::new(reflect);
    graphics.0.cached.shaders.insert(
        (snippets.clone(), file.to_string()),
        (raw.clone(), reflected.clone()),
    );
    log::debug!("Saved shader module for {file} in graphics cache");

    // Return the compiled wgpu module and the reflected mdule
    Ok((raw, reflected))
}

// Load a function module and convert it to a ResolvedInclude
fn load_function_module(
    path: &str,
    assets: &Assets,
) -> Result<shaderc::ResolvedInclude, String> {
    // Make sure the path is something we can load (.glsl file)
    let pathbuf = PathBuf::try_from(path).unwrap();

    // Load the path from the asset manager
    let content = assets
        .load::<FunctionModule>(pathbuf.as_os_str().to_str().unwrap())
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
    snippets: &Snippets,
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
    included: &Included,
) -> Result<shaderc::ResolvedInclude, String> {
    // If we're too deep, assume that the user caused a cyclic reference, and return an error
    if depth > 40 {
        return Err(format!("Include cyclic reference detected"));
    }

    // Check if the user wants to load a snippet or asset
    // If it's a snippet, then the name of the snippet should be surrounded with ""
    // If it's an asset, then the name of the file should be surrounded with <>
    let resembles = matches!(_type, shaderc::IncludeType::Standard);

    // Check if this file/snippet was already loaded before
    let mut locked = included.lock();
    if locked.contains(target) {
        return Ok(shaderc::ResolvedInclude {
            resolved_name: target.to_string(),
            content: "".to_string(),
        });
    }

    // Either load it as an asset or a snippet
    let output = if resembles {
        log::debug!("Loading shader function module '{target}'");
        load_function_module(&target, assets)
    } else {
        log::debug!("Loading shader source snippet '{target}'");
        load_snippet(&target, snippets)
    };

    // Convert the error to a string instead
    locked.insert(target.to_string());
    output
}

// This is a compiled shader module that we can use in multiple pipelines
// We can clone this shader module since we should be able to share them
pub struct Compiled<M: ShaderModule> {
    // Wgpu module and spirv reflected module
    raw: Arc<wgpu::ShaderModule>,
    reflected: Arc<spirq::EntryPoint>,

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
            reflected: self.reflected.clone(),
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

    // Get the underlying raw reflected module
    pub fn reflected(&self) -> &spirq::EntryPoint {
        &self.reflected
    }

    // Get the shader module name for this module
    pub fn name(&self) -> &str {
        &self.name
    }
}
