use crate::{
    BindResourceType, Buffer, Extent, FunctionModule, GpuPod, GpuPodInfo, Graphics, ModuleKind,
    ModuleVisibility, PushConstantLayout, ReflectedShader, Region, ShaderCompilationError,
    ShaderError, ShaderModule, ShaderReflectionError, SpecConstant, StorageAccess, Texel,
    TexelInfo, Texture, VertexModule, ViewDimension,
};
use ahash::{AHashMap, AHashSet};
use assets::Assets;
use itertools::Itertools;
use parking_lot::Mutex;
use snailquote::unescape;
use std::{
    any::TypeId,
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant, hash::{Hash, Hasher},
};
use thiserror::Error;
use vek::serde::__private::de;

use super::create_pipeline_layout;

// Reason why these are BTreeMaps is because we need to be able to hash them
pub(crate) type Snippets = BTreeMap<String, String>;
pub(crate) type Defines = BTreeMap<String, String>;
pub(crate) type Constants = BTreeMap<u32, SpecConstant>;

// Type alias for snippets and resources
pub(crate) type ResourceBindingTypes = AHashMap<String, BindResourceType>;
pub(crate) type MaybePushConstantLayout = Option<PushConstantLayout>;
pub(crate) type Included = Arc<Mutex<AHashSet<String>>>;

// Contains the source code of the file and defines used
pub(crate) type CachedSpirvKey = (String, Defines, Snippets);

// FIXME: This is also supposed to take in the value of the spec-constants but since they contain a variant that stores a f32 they can't be hashed
pub(crate) type CachedShaderKey = (String, Defines, Snippets);

pub type OptimizationLevel = shaderc::OptimizationLevel;

// This is a compiler that will take GLSL code and create a WGPU module
// This compiler also allows us to define constants and snippets before compilation
// This compiler will be used within the Shader and ComputeShader to compile the modules in batch
pub struct Compiler<'a> {
    pub(crate) assets: &'a Assets,
    pub(crate) graphics: &'a Graphics,
    pub(crate) snippets: Snippets,
    pub(crate) constants: Constants,
    pub(crate) defines: Defines,
    pub(crate) resource_types: ResourceBindingTypes,
    pub(crate) maybe_push_constant_layout: MaybePushConstantLayout,
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
            defines: Default::default(),
        }
    }

    // Set the value of a specilization constant within the shader
    pub fn use_constant(&mut self, specid: u32, value: impl Into<SpecConstant>) {
        self.constants.insert(specid, value.into());
    }

    // Include a snippet directive that will replace #includes surrounded by ""
    pub fn use_snippet(&mut self, name: impl ToString, value: impl ToString) {
        self.snippets.insert(name.to_string(), value.to_string());
    }

    // Set the value of a "#define" pre-processor macro
    // A define is different than a snippet in that you do not load it within the shader
    // It automatically gets added to the top of the shader
    pub fn use_define(&mut self, name: impl ToString, value: impl ToString) {
        self.defines.insert(name.to_string(), value.to_string());
    }

    // Convert the given GLSL code to SPIRV code, then compile said SPIRV code
    // This uses the defined resoures defined in this compiler
    pub(crate) fn compile<M: ShaderModule>(&self, module: M) -> Result<Compiled<M>, ShaderError> {
        // Decompose the module into file name and source
        let (path, source) = module.into_raw_parts();

        // Compile GLSL to SPIRV then to Wgpu
        let time = std::time::Instant::now();
        let (raw, reflected) = compile(
            &M::kind(),
            &self.graphics,
            &self.assets,
            &self.snippets,
            &self.constants,
            &self.defines,
            source,
            &path,
        )
        .map_err(ShaderError::Compilation)?;
        log::debug!(
            "Compiled shader {path:?} sucessfully! Took {}ms",
            time.elapsed().as_millis()
        );

        Ok(Compiled {
            raw,
            snippets: self.snippets.clone(),
            path: path.into(),
            _phantom: PhantomData,
            graphics: self.graphics.clone(),
            reflected,
            checksum: 0,
        })
    }

    // Convert the given shader modules
    pub(crate) fn create_pipeline_layout(
        &self,
        names: &[&str],
        modules: &[&spirq::EntryPoint],
        visibility: &[ModuleVisibility],
    ) -> Result<(Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>), ShaderError> {
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
    pub fn use_resource_type(&mut self, name: impl ToString, resource: BindResourceType) {
        let name = name.to_string();
        match self.resource_types.entry(name.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                log::debug!("Binding resource '{name}' was replaced");
                occupied.insert(resource);
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(resource);
            }
        }
    }

    // Define a uniform buffer type's inner struct type
    pub fn use_uniform_buffer<T: GpuPod>(&mut self, name: impl ToString) {
        let size = T::size();
        self.use_resource_type(name, BindResourceType::UniformBuffer { size });
    }

    // Define a storage buffer type's inner struct type
    pub fn use_storage_buffer<T: GpuPod>(&mut self, name: impl ToString, access: StorageAccess) {
        let size = T::size();
        self.use_resource_type(name, BindResourceType::StorageBuffer { size, access });
    }

    // Define a uniform sampled texture's type and texel
    pub fn use_sampled_texture<T: Texture>(&mut self, name: impl ToString) {
        let sampler_name = format!("{}_sampler", name.to_string());
        self.use_sampler::<T::T>(sampler_name);

        let dimensionality = <T::Region as Region>::view_dimension();
        let info = <T::T as Texel>::info();
        let format = info.format();

        self.resource_types.insert(
            name.to_string(),
            BindResourceType::SampledTexture {
                format,
                sample_type: super::map_texture_sample_type(&self.graphics, info),
                sampler_binding: super::map_sampler_binding_type(&self.graphics, info),
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
                sampler_binding: super::map_sampler_binding_type(&self.graphics, info),
            },
        );
    }

    // Define a storage texture that we can read / write to
    pub fn use_storage_texture<T: Texture>(&mut self, name: impl ToString, access: StorageAccess) {
        let dimensionality = <T::Region as Region>::view_dimension();
        let info = <T::T as Texel>::info();
        let format = info.format();

        self.resource_types.insert(
            name.to_string(),
            BindResourceType::StorageTexture {
                access,
                format,
                view_dimension: dimensionality,
            },
        );
    }

    // Define a push constant range to be pushed
    pub fn use_push_constant_layout(&mut self, layout: PushConstantLayout) {
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
    defines: &Defines,
    mut source: String,
    path: &Path,
) -> Result<(Arc<wgpu::ShaderModule>, Arc<spirq::EntryPoint>), ShaderCompilationError> {
    // Check if the shader module was already compiled and WGPU created it
    let key = (source.clone(), defines.clone(), snippets.clone());

    /*
    // If the shader cache already contains the compiled shader, simply reuse it
    if let Some(value) = graphics
        .0
        .cached
        .shaders
        .get(&(snippets.clone(), path.to_path_buf()))
    {
        let (raw, reflected) = value.value();
        log::debug!("Found shader module in cache for {path:?}, using it...");
        return Ok((raw.clone(), reflected.clone()));
    } else {
        log::warn!("Did not find cached shader module for {path:?}");
    }
    */

    // Compile SPIRV if it was not in cache already
    let key = (source.clone(), defines.clone(), snippets.clone());
    let cached = graphics.0.cached.spirvs.get(&key);
    let spirv = if cached.is_none() {
        Some(compile_spirv(
            path,
            source,
            defines,
            snippets,
            assets,
            graphics,
            kind
        )?)
    } else {
        None
    };

    // Fetch cached SPIRV binary if it was already compiled
    let mut spirv = spirv.as_ref().map(|x| x.as_binary()).unwrap_or_else(|| &cached.as_ref().unwrap()).to_vec();
    let before = spirv.clone();

    // Cache the SPIRV into the shader cache if needed
    if cached.is_none() {
        //graphics.0.cached.spirvs.insert(key, spirv.clone());
    }

    // Parse the spirv manually to be able to handle specialization constants
    specialize_spec_constants(&mut spirv, &constants);

    // Setup basic config spirq option
    let mut reflect = spirq::ReflectConfig::new()
        .spv(before)
        .combine_img_samplers(false)
        .ref_all_rscs(true)
        .gen_unique_names(false)
        .reflect()
        .unwrap();
    assert!(reflect.len() == 1);
    let reflect = reflect.pop().unwrap();

    // Compile the Wgpu shader (raw spirv passthrough)
    let wgpu = unsafe {
        graphics
            .device()
            .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: Some(&format!("shader-module-{path:?}")),
                source: wgpu::util::make_spirv_raw(bytemuck::cast_slice(&spirv)),
            })
    };

    // Cache the results of the shader compilation
    let raw = Arc::new(wgpu);
    let reflected = Arc::new(reflect);
    /*
    graphics.0.cached.shaders.insert(
        checksum,
        (raw.clone(), reflected.clone()),
    );
    */
    //log::debug!("Saved shader module for {file} in graphics cache");

    // Return the compiled wgpu module and the reflected mdule
    Ok((raw, reflected))
}

// Force the compilation of SPIRV code
// Only gets executed if the SPIRV was not cached in the shader cache
fn compile_spirv(
    path: &Path,
    source: String,
    defines: &Defines,
    snippets: &Snippets,
    assets: &Assets,
    graphics: &Graphics,
    kind: &ModuleKind
) -> Result<shaderc::CompilationArtifact, ShaderCompilationError> {
    let file = path.file_name().unwrap().to_str().unwrap();
    let version_line_index = source
        .lines()
        .position(|x| x.starts_with("#version"))
        .unwrap();
    let mut lines = source
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    for (name, define) in defines {
        lines.insert(version_line_index + 1, format!("#define {name} {define}\n"));
    }

    let extensions = ["GL_EXT_samplerless_texture_functions"];
    for ext in extensions {
        lines.insert(
            version_line_index + 1,
            format!("#extension {ext} : require\n"),
        );
    }
    let source = lines.join("\n");
    
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_invert_y(false);
    
    let included = Included::default();
    options.set_include_callback(move |target, _type, current, depth| {
        include(current, _type, target, depth, assets, &snippets, &included)
    });
    
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
            shaderc::Error::CompilationError(_, value) => {
                let source = source
                    .lines()
                    .enumerate()
                    .map(|(count, line)| format!("({}): {}", count + 1, line))
                    .collect::<Vec<String>>()
                    .join("\n");

                log::error!("Failed compilation of shader '{file}'");
                log::error!("Source code: \n\n{source}\n\n");

                for line in value.lines() {
                    log::error!("{}", line);
                }

                ShaderCompilationError::ShaderC
            }
            _ => todo!(),
        })?;
    if !artifact.get_warning_messages().is_empty() {
        log::warn!("ShaderC warning: {}", artifact.get_warning_messages());
    }
    Ok(artifact)
}

// Specialize spec constants ourselves cause there's no other way to do it (fuck)
fn specialize_spec_constants(binary: &mut [u32], constants: &Constants) {
    // Converts a SpecConstant op code to it's specialized variant (Constant)
    fn specialize(op_code_index: usize, binary: &mut [u32], defined: SpecConstant) {
        // Get the op code of the spec constant
        let op_code = binary[op_code_index] & 0x0000ffff;

        // Get the index of the spec constant literal
        let literal_index = match op_code {
            48 | 49 => op_code_index + 2,
            50 => op_code_index + 3,
            _ => panic!(),
        };

        // Write to the literal value if it's not a boolean
        let literal = &mut binary[literal_index];
        *literal = match defined {
            SpecConstant::I32(val) => bytemuck::cast(val),
            SpecConstant::U32(val) => val,
            SpecConstant::F32(val) => bytemuck::cast(val),
            _ => *literal,
        };

        // Update the OpCode of the spec constant to the proper one in case it's a boolean
        let new = match op_code {
            48 | 49 => {
                if let SpecConstant::BOOL(val) = defined {
                    match val {
                        true => 48,
                        false => 49,
                    }
                } else {
                    panic!()
                }
            }
            x => x,
        };

        // Write new op code heheheha
        binary[op_code_index] &= 0xffff0000;
        binary[op_code_index] |= new;
    }

    // List of op codes that we must change
    let spec_consts_op_codes = [48u32, 49, 50];

    // Contains the SpecId decorations for each constant
    let mut spec_ids = AHashMap::<u32, u32>::default();

    // Loop till we find an OpSpecConstant

    // TODO: Atm this will just look into the spirv and find words that match up with the predefined ones
    // Very unsafe cause it could lead to corrupted data. Must find a way to make it safe
    for i in 0..binary.len() {
        // Get the op code and argument count
        let word = binary[i];
        let op = word & 0x0000ffff;
        let count = (word & 0xffff0000) >> 16;

        // Keep track of SpecId decorations
        if op == 71 && count == 4 && binary[i + 2] == 1 {
            spec_ids.insert(binary[i + 1], binary[i + 3]);
        }

        // For now, we only support 32 bit types
        if spec_consts_op_codes.contains(&op) && (count == 4 || count == 3) {
            // Get the literal value that this spec constant is defaulted to
            let id = binary[i + 2];
            let spec_id = spec_ids.get(&id).unwrap();

            // Get the value specified by the user
            if let Some(value) = constants.get(spec_id) {
                specialize(i, binary, *value);
            }
        }
    }
}

// Load a function module and convert it to a ResolvedInclude
fn load_function_module(path: &str, assets: &Assets) -> Result<shaderc::ResolvedInclude, String> {
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
fn load_snippet(name: &str, snippets: &Snippets) -> Result<shaderc::ResolvedInclude, String> {
    let snippet = snippets
        .get(name)
        .ok_or(format!("Snippet {} was not defined", name.to_string()))?;
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
        log::debug!("{target} was already loaded, no need to load it again");
        return Ok(shaderc::ResolvedInclude {
            resolved_name: target.to_string(),
            content: "".to_string(),
        });
    }

    // Either load it as an asset or a sniponst uinpet
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
    checksum: u32,
    snippets: Snippets,

    // Helpers
    path: Arc<Path>,
    _phantom: PhantomData<M>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<M: ShaderModule> Clone for Compiled<M> {
    fn clone(&self) -> Self {
        Self {
            snippets: self.snippets.clone(),
            raw: self.raw.clone(),
            path: self.path.clone(),
            _phantom: self._phantom.clone(),
            graphics: self.graphics.clone(),
            reflected: self.reflected.clone(),
            checksum: self.checksum,
        }
    }
}

impl<M: ShaderModule> Drop for Compiled<M> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.raw) == 2 {
            /*
            let path = self.path.as_ref();
            let path = PathBuf::from(path);
            assert!(self.graphics.drop_cached_shader_module(self.checksum));
            */
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

    // Get the shader module path for this module
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Get the shader module name for this module
    pub fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}
