use std::{hash::Hash, num::NonZeroU32, sync::Arc};

use crate::{
    visibility_to_wgpu_stage, BufferValidationError, Compiled, ComputeModule, ElementType,
    FragmentModule, Graphics, ModuleKind, ModuleVisibility, PushConstantValidationError,
    SamplerValidationError, ShaderModule, ShaderReflectionError, TexelChannels, TexelInfo,
    TextureValidationError, VertexModule,
};
use ahash::{AHashMap, AHashSet};
use arrayvec::ArrayVec;
use spirq::DescriptorType;
use utils::enable_in_range;
use wgpu::TextureFormatFeatureFlags;

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedShader {
    pub taken_bind_group_layouts: usize,
    pub bind_group_layouts: Vec<Option<BindGroupLayout>>,
    pub push_constant_layout: Option<PushConstantLayout>,
}

// A bind group contains one or more bind entries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupLayout {
    pub bind_entry_layouts: Vec<BindResourceLayout>,
}

// A binding entry is a single binding resource from within a group
// Eg. a uniform buffer, a sampler, a texture, or storage buffer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindResourceLayout {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub count: Option<NonZeroU32>,
    pub resource_type: BindResourceType,
    pub visibility: ModuleVisibility,
}

// Visiblity for the set push constants bitset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PushConstantLayout {
    Single(NonZeroU32, ModuleVisibility),
    SplitVertexFragment {
        vertex: NonZeroU32,
        fragment: NonZeroU32,
    },
}

impl PushConstantLayout {
    // Create a push constant layout for a single module or SharedVG modules
    pub fn single(size: usize, visibility: ModuleVisibility) -> Option<Self> {
        let size = NonZeroU32::new(size as u32)?;
        Some(Self::Single(size, visibility))
    }

    // Create a push constant layout for a vertex module
    pub fn vertex(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Vertex)
    } 

    // Create a push constant layout for a fragment module
    pub fn fragment(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Fragment)
    } 

    // Create a push constant layout for a compute module
    pub fn compute(size: usize) -> Option<Self> {
        Self::single(size, ModuleVisibility::Compute)
    }     


    // Create a push constant layout for split vertex / fragment modules
    pub fn split(vertex: usize, fragment: usize) -> Option<Self> {
        let vertex = NonZeroU32::new(vertex as u32)?;
        let fragment = NonZeroU32::new(fragment as u32)?;
        Some(Self::SplitVertexFragment { vertex, fragment })
    }

    // Convert this push constant layout to it's ModuleVisibility
    pub fn visibility(&self) -> ModuleVisibility {
        match self {
            PushConstantLayout::Single(_, visibility) => *visibility,
            PushConstantLayout::SplitVertexFragment { .. } => ModuleVisibility::VertexFragment,
        }
    }

    // Get the MAX size required to set the push constant bytes
    pub fn size(&self) -> NonZeroU32 {
        match self {
            PushConstantLayout::Single(x, _) => *x,
            PushConstantLayout::SplitVertexFragment { vertex, fragment } => {
                NonZeroU32::new(vertex.get() + fragment.get()).unwrap()
            }
        }
    }
}

// Accessing types for storage buffer and storage resources
pub type StorageAccess = spirq::AccessType;

// The type of BindingEntry. This is fetched from the given
// For now, only buffers, samplers, and texture are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindResourceType {
    // A uniform buffer
    UniformBuffer {
        size: usize,
    },

    // A storage buffer
    StorageBuffer {
        size: usize,
        access: StorageAccess,
    },

    // A sampler type that we can use to sample textures (sampler2D)
    Sampler {
        format: wgpu::TextureFormat,
        sampler_binding: wgpu::SamplerBindingType,
    },

    // A sampled texture (texture2D with separate sampler)
    SampledTexture {
        format: wgpu::TextureFormat,
        sample_type: wgpu::TextureSampleType,
        sampler_binding: wgpu::SamplerBindingType,
        view_dimension: wgpu::TextureViewDimension,
    },

    // A storage texture
    StorageTexture {
        access: StorageAccess,
        format: wgpu::TextureFormat,
        view_dimension: wgpu::TextureViewDimension,
    },
}

// Internal data that is passed to module creation
// This is used by the compiler to define formats and types used by shaders
struct InternalDefinitions<'a> {
    resource_binding_types: &'a super::ResourceBindingTypes,
    maybe_push_constant_layout: &'a super::MaybePushConstantLayout,
}

pub(crate) const UNIFORM_BUFFER_STRINGIFIED_NAME: &'static str = "Uniform Buffer";
pub(crate) const STORAGE_BUFFER_STRINGIFIED_NAME: &'static str = "Storage Buffer";
pub(crate) const SAMPLED_TEXTURE_STRINGIFIED_NAME: &'static str = "Sampled Texture";
pub(crate) const STORAGE_TEXTURE_STRINGIFIED_NAME: &'static str = "Storage Texture";
pub(crate) const SAMPLER_STRINGIFIED_NAME: &'static str = "Sampler";

// Convert an bind resource type enum into a variant name
pub(crate) fn stringify_bind_resource_type(val: &BindResourceType) -> &'static str {
    match val {
        BindResourceType::UniformBuffer { .. } => UNIFORM_BUFFER_STRINGIFIED_NAME,
        BindResourceType::StorageBuffer { .. } => STORAGE_BUFFER_STRINGIFIED_NAME,
        BindResourceType::Sampler { .. } => SAMPLER_STRINGIFIED_NAME,
        BindResourceType::SampledTexture { .. } => SAMPLED_TEXTURE_STRINGIFIED_NAME,
        BindResourceType::StorageTexture { .. } => STORAGE_TEXTURE_STRINGIFIED_NAME,
    }
}

// Convert a reflected bind entry layout to a wgpu binding type
pub(super) fn map_binding_type(value: &BindResourceLayout) -> wgpu::BindingType {
    match value.resource_type {
        BindResourceType::UniformBuffer { size } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },

        BindResourceType::StorageBuffer { size, access } => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage {
                read_only: if let StorageAccess::ReadOnly = access {
                    true
                } else {
                    false
                },
            },
            has_dynamic_offset: false,
            min_binding_size: None,
        },

        BindResourceType::Sampler {
            sampler_binding, ..
        } => wgpu::BindingType::Sampler(sampler_binding),

        BindResourceType::SampledTexture {
            sample_type,
            view_dimension,
            ..
        } => wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled: false,
        },

        BindResourceType::StorageTexture {
            format,
            access,
            view_dimension,
        } => wgpu::BindingType::StorageTexture {
            access: match access {
                spirq::AccessType::ReadOnly => wgpu::StorageTextureAccess::ReadOnly,
                spirq::AccessType::WriteOnly => wgpu::StorageTextureAccess::WriteOnly,
                spirq::AccessType::ReadWrite => wgpu::StorageTextureAccess::ReadWrite,
            },
            format,
            view_dimension,
        },
    }
}

// Create a wgpu TextureSampleType out of some basic TexelInfo
pub(super) fn map_texture_sample_type(
    graphics: &Graphics,
    info: TexelInfo,
) -> wgpu::TextureSampleType {
    match info.element() {
        ElementType::Eight {
            signed,
            normalized: false,
        }
        | ElementType::Sixteen {
            signed,
            normalized: false,
        }
        | ElementType::ThirtyTwo { signed } => match signed {
            true => wgpu::TextureSampleType::Sint,
            false => wgpu::TextureSampleType::Uint,
        },

        ElementType::FloatSixteen
        | ElementType::Eight {
            normalized: true, ..
        }
        | ElementType::Sixteen {
            normalized: true, ..
        }
        | ElementType::FloatThirtyTwo => {
            let adapter = graphics.adapter();
            let format = info.format();
            let flags = adapter.get_texture_format_features(format).flags;

            let depth = matches!(info.channels(), TexelChannels::Depth);

            // TODO: Pretty sure this is wrong

            if flags.contains(TextureFormatFeatureFlags::FILTERABLE) && !depth {
                wgpu::TextureSampleType::Float { filterable: true }
            } else {
                wgpu::TextureSampleType::Depth
            }
        }

        ElementType::Compressed(_) => todo!(),
    }
}

// Create a wgpu SamplerBindingType out of some basic TexelInfo
pub(super) fn map_sampler_binding_type(
    graphics: &Graphics,
    info: TexelInfo,
) -> wgpu::SamplerBindingType {
    let adapter = graphics.adapter();
    let format = info.format();
    let flags = adapter.get_texture_format_features(format).flags;

    let depth = matches!(info.channels(), TexelChannels::Depth);

    if flags.contains(TextureFormatFeatureFlags::FILTERABLE) && !depth {
        wgpu::SamplerBindingType::Filtering
    } else if depth {
        wgpu::SamplerBindingType::Comparison
    } else {
        wgpu::SamplerBindingType::NonFiltering
    }
}

// Convert a spirv dimension into a wgpu TextureViewDimension
pub(super) fn map_spirv_dim(dim: spirv::Dim, array: bool) -> wgpu::TextureViewDimension {
    match (dim, array) {
        (spirv::Dim::Dim1D, false) => wgpu::TextureViewDimension::D1,
        (spirv::Dim::Dim2D, true) => wgpu::TextureViewDimension::D2Array,
        (spirv::Dim::Dim2D, false) => wgpu::TextureViewDimension::D2,
        (spirv::Dim::Dim3D, false) => wgpu::TextureViewDimension::D3,
        (spirv::Dim::DimCube, true) => wgpu::TextureViewDimension::CubeArray,
        (spirv::Dim::DimCube, false) => wgpu::TextureViewDimension::Cube,
        _ => panic!("Not supported "),
    }
}

// Convert a spirv format into a wgpu TextureFormat
pub(super) fn map_spirv_format(format: spirv::ImageFormat) -> wgpu::TextureFormat {
    use spirv::ImageFormat as F;
    use wgpu::TextureFormat as T;

    match format {
        F::Rgba32f => T::Rgba32Float,
        F::Rgba16f => T::Rgba16Float,
        F::R32f => T::R32Float,
        F::Rgba8 => T::Rgba8Unorm,
        F::Rgba8Snorm => T::Rgba8Snorm,
        F::Rg32f => T::Rg32Float,
        F::Rg16f => T::Rg16Float,
        F::R11fG11fB10f => T::Rg11b10Float,
        F::R16f => T::R16Float,
        F::Rgba16 => T::Rgba16Unorm,
        F::Rgb10A2 => T::Rgb10a2Unorm,
        F::Rg16 => T::Rg16Unorm,
        F::Rg8 => T::Rg8Unorm,
        F::R16 => T::R16Unorm,
        F::R8 => T::R8Unorm,
        F::Rgba16Snorm => T::Rgba16Snorm,
        F::Rg16Snorm => T::Rg16Snorm,
        F::Rg8Snorm => T::Rg8Snorm,
        F::R16Snorm => T::R16Snorm,
        F::R8Snorm => T::R8Snorm,
        F::Rgba32i => T::Rgba32Sint,
        F::Rgba16i => T::Rgba16Sint,
        F::Rgba8i => T::Rgba32Sint,
        F::R32i => T::R32Sint,
        F::Rg32i => T::Rg32Sint,
        F::Rg16i => T::Rg16Sint,
        F::Rg8i => T::Rg8Sint,
        F::R16i => T::R16Sint,
        F::R8i => T::R8Sint,
        F::Rgba32ui => T::Rgba32Uint,
        F::Rgba16ui => T::Rgba16Uint,
        F::Rgba8ui => T::Rgba8Uint,
        F::R32ui => T::R32Uint,
        //F::Rgb10a2ui => T::Rgb10,
        F::Rg32ui => T::Rg32Uint,
        F::Rg16ui => T::Rg16Uint,
        F::Rg8ui => T::Rg8Uint,
        F::R16ui => T::R16Uint,
        F::R8ui => T::R8Uint,
        _ => panic!("Not supported"),
    }
}

// Convert a spirq scalar type into a wgpu sample type
pub(super) fn map_spirv_scalar_type(
    scalar_type: spirq::ty::ScalarType,
    format: wgpu::TextureFormat,
) -> wgpu::TextureSampleType {
    match scalar_type {
        spirq::ty::ScalarType::Signed(_) => wgpu::TextureSampleType::Sint,
        spirq::ty::ScalarType::Unsigned(_) => wgpu::TextureSampleType::Uint,
        spirq::ty::ScalarType::Float(_) => match format {
            wgpu::TextureFormat::Depth16Unorm
            | wgpu::TextureFormat::Depth24Plus
            | wgpu::TextureFormat::Depth24PlusStencil8
            | wgpu::TextureFormat::Depth32Float
            | wgpu::TextureFormat::Depth32FloatStencil8 => {
                wgpu::TextureSampleType::Depth
            }
            _ => wgpu::TextureSampleType::Float { filterable: true },
        },
        _ => panic!("Not supported"),
    }
}

// Get the size of a spirq type
// Returns stride if it is a dynamic array
pub(super) fn get_spirq_type_size(_type: &spirq::ty::Type) -> Option<usize> {
    use spirq::ty::Type;

    //dbg!(_type.nbyte());

    match _type {
        Type::Scalar(_) | Type::Vector(_) | Type::Matrix(_) => Some(_type.nbyte().unwrap()),
        Type::Array(array) => Some(if array.nrepeat.is_some() {
            array.stride.unwrap()
        } else {
            array.nbyte()
        }),
        Type::Struct(structure) => {
            let size = _type.nbyte().unwrap();

            // Returns true if the last element has a valid size
            let last = structure
                .members
                .last()
                .map(|x| x.ty.nbyte().unwrap_or(0) > 0)
                .unwrap_or_default();

            // Check if it is a dynamically sized array
            if size == 0 && structure.members.len() == 1 && !last {
                if let Type::Array(array) = &structure.members[0].ty {
                    Some(array.stride.unwrap())
                } else {
                    panic!()
                }
            } else if last {
                // No dynamic elements, always valid
                Some(size)
            } else {
                // Neither dynamic or static... wtf?
                panic!()
            }
        }
        _ => None,
    }
}

// Create a pipeline layout for a combination of shader modules using the specified definitions
pub(super) fn create_pipeline_layout(
    graphics: &Graphics,
    names: &[&str],
    modules: &[&spirq::EntryPoint],
    visibility: &[ModuleVisibility],
    resource_binding_types: &super::ResourceBindingTypes,
    maybe_push_constant_layout: &super::MaybePushConstantLayout,
) -> Result<(Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>), ShaderReflectionError> {
    // Make sure the local workgroup size limit is respected in case of a compute shader
    if visibility[0] == ModuleVisibility::Compute {
        let adapter = graphics.adapter();
        let limit = adapter.limits().max_compute_invocations_per_workgroup;
        let module = modules[0];
        let consts = &module.exec_modes;
        let first = &consts[0];
        assert!(first.exec_mode == spirv::ExecutionMode::LocalSize);
        assert!(first.operands.len() == 3);
        let mul = (0..3)
            .into_iter()
            .map(|i| first.operands[i].value.to_u32())
            .product::<u32>();

        if mul >= limit {
            return Err(
                ShaderReflectionError::ComputeShaderLocalWorkgroupSizeLimit { shader: mul, limit },
            );
        }
    }

    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: Vec<Option<AHashMap<u32, BindResourceLayout>>> = (0..4).map(|_| None).collect();

    // Return error if the user defined a push constant that is greater than the device size
    // or if there isn't push constants for the specified module in the shaders
    if let Some(push_constant_layout) = maybe_push_constant_layout {
        // We always assume that the "visibility" array given is either [Vertex, Fragment] or [Compute]
        // Under that assumption, all we really need to check is the compute module and if it contains the proper push constant
        let compute = matches!(visibility[0], ModuleVisibility::Compute);
        match (push_constant_layout.visibility(), compute) {
            (ModuleVisibility::Vertex, false) => {}
            (ModuleVisibility::Fragment, false) => {}
            (ModuleVisibility::VertexFragment, false) => {}
            (ModuleVisibility::Compute, true) => {}
            _ => {
                return Err(ShaderReflectionError::PushConstantValidation(
                    PushConstantValidationError::PushConstantVisibilityIntersect,
                ))
            }
        }

        // Check size and make sure it's valid
        let defined = push_constant_layout.size().get();
        let max = graphics.device().limits().max_push_constant_size;
        if defined > max {
            return Err(ShaderReflectionError::PushConstantValidation(
                PushConstantValidationError::PushConstantSizeTooBig(defined, max),
            ));
        }

        // Get the push constant sizes as defined in the shader modules
        let mut iter = modules.iter().flat_map(|module| {
            module.vars.iter().filter_map(|x| match x {
                spirq::Variable::PushConstant { ty, .. } => ty.nbyte().map(|x| x as u32),
                _ => None,
            })
        });
        let (first, second) = (iter.next(), iter.next());

        // Validate the push constants and check if they were defined properly in the shader
        let valid = match push_constant_layout {
            PushConstantLayout::Single(size, _) => {
                if second.is_none() {
                    Some(size.get()) == first
                } else {
                    todo!()
                }
            }
            PushConstantLayout::SplitVertexFragment { vertex, fragment } => {
                if let (Some(first), Some(second)) = (first, second) {
                    // Assumes the vertex and fragment push constants are tightly packed together
                    vertex.get() == first && fragment.get() == (second - first)
                } else {
                    todo!()
                }
            }
        };

        // Return error if not defined
        if !valid {
            return Err(ShaderReflectionError::PushConstantValidation(
                PushConstantValidationError::PushConstantNotDefinedOrDiffSized,
            ));
        }
    }

    let definitions = InternalDefinitions {
        resource_binding_types,
        maybe_push_constant_layout,
    };

    // Iterate through the sets and create the bind groups for each of the resources
    for (index, module) in modules.iter().enumerate() {
        let visibility = visibility[index];

        // Create a hashmap that contains all the variables per group
        let mut sets = AHashMap::<u32, Vec<spirq::Variable>>::default();

        let iter = module.vars.iter().filter_map(|variable| {
            if let spirq::Variable::Descriptor { desc_bind, .. } = variable {
                Some((desc_bind.set(), variable))
            } else {
                None
            }
        });

        // Add the binded variables to the hashmap
        for (set, variable) in iter {
            let vec = sets.entry(set).or_default();
            vec.push(variable.clone());
        }

        for set in 0..4 {
            // Skip this set if no variables are defined within it
            if !sets.contains_key(&set) {
                continue;
            }

            // Iterate over the descriptor bindings
            for variable in sets.get(&set).unwrap() {
                let spirq::Variable::Descriptor { name, desc_bind, desc_ty, ty, nbind } = variable else {
                    panic!()
                };

                if name.is_none() {
                    continue;
                }

                // Get the name, binding, and required values for this var
                let name = name.as_ref().unwrap().clone();
                let binding = desc_bind.bind();

                // Get the merged group layout and merged group entry layouts
                let merged_group_layout = &mut groups[set as usize];
                let merged_group_entry_layouts =
                    merged_group_layout.get_or_insert_with(|| Default::default());

                // Make sure the resource is defined within the compiler
                let resource = definitions
                    .resource_binding_types
                    .get(&name)
                    .ok_or(ShaderReflectionError::NotDefinedInCompiler(name.clone()))?;

                // Get the binding type for this global variable
                let binding_type = match desc_ty {
                    // Reflect sampler type
                    DescriptorType::Sampler() => {
                        Some(reflect_sampler(&resource).map_err(|error| {
                            ShaderReflectionError::SamplerValidation {
                                resource: name.clone(),
                                error,
                            }
                        }))
                    }

                    // Reflect sampled texture type
                    DescriptorType::SampledImage() => Some(
                        reflect_sampled_texture(
                            &name,
                            graphics,
                            &resource,
                            ty.as_sampled_img().unwrap(),
                        )
                        .map_err(|error| {
                            ShaderReflectionError::TextureValidation {
                                resource: name.clone(),
                                error,
                            }
                        }),
                    ),

                    // Reflect storage texture type
                    DescriptorType::StorageImage(access) => Some(
                        reflect_storage_texture(
                            &name,
                            graphics,
                            &resource,
                            access,
                            ty.as_storage_img().unwrap(),
                        )
                        .map_err(|error| {
                            ShaderReflectionError::TextureValidation {
                                resource: name.clone(),
                                error,
                            }
                        }),
                    ),

                    // Reflect uniform buffer type
                    DescriptorType::UniformBuffer() => {
                        Some(reflect_uniform_buffer(&resource, ty).map_err(|error| {
                            ShaderReflectionError::BufferValidation {
                                resource: name.clone(),
                                error,
                            }
                        }))
                    }

                    // Reflect storage buffer type
                    DescriptorType::StorageBuffer(access) => Some(
                        reflect_storage_buffer(&resource, access, ty).map_err(|error| {
                            ShaderReflectionError::BufferValidation {
                                resource: name.clone(),
                                error,
                            }
                        }),
                    ),

                    _ => None,
                };

                // If none, ignore
                let Some(resource_type) = binding_type else {
                    continue
                };

                // Extract error if needed
                let resource_type = resource_type?;

                // Create a bind entry layout
                let bind_entry_layout = BindResourceLayout {
                    name: name.to_string(),
                    binding,
                    group: set,
                    resource_type,
                    visibility,
                    count: (*nbind > 1).then(|| NonZeroU32::new(*nbind - 1).unwrap()),
                };

                // Merge each entry for this group individually
                match merged_group_entry_layouts.entry(binding) {
                    // Merge an already existing layout with the new one
                    std::collections::hash_map::Entry::Occupied(mut occupied) => {
                        let merged_bind_entry_layout = occupied.get_mut();
                        let old = bind_entry_layout;
                        let merged = merged_bind_entry_layout;

                        // Make sure the currently merged layout and the new layout
                        // have well... the same layout
                        if old.resource_type != merged.resource_type {
                            log::warn!("{:?}", old.resource_type);
                            log::warn!("{:?}", merged.resource_type);
                            panic!("Not the same layout");
                        }

                        // Merge the visibility to allow more modules to access this entry
                        merged.visibility.try_insert(old.visibility).unwrap();
                    }

                    // If the spot is vacant, add the bind entry layout for the first time
                    std::collections::hash_map::Entry::Vacant(vacant) => {
                        vacant.insert(bind_entry_layout.clone());
                    }
                }
            }
        }
    }

    // Convert the hashmaps that contain the bind resource layouts to arrays
    let bind_group_layouts = groups
        .into_iter()
        .map(|hashmap| {
            hashmap.map(|hashmap| {
                let bind_entry_layouts = hashmap
                    .into_iter()
                    .map(|(_, entry)| entry)
                    .collect::<Vec<_>>();

                BindGroupLayout { bind_entry_layouts }
            })
        })
        .collect::<Vec<Option<BindGroupLayout>>>();

    // Calculate the number of group layouts that we must use (starting from the back)
    let taken_bind_group_layouts = bind_group_layouts
        .iter()
        .rposition(|x| x.is_some())
        .map(|x| x + 1)
        .unwrap_or_default();

    // Create a reflected shader with the given compiler params
    let shader = ReflectedShader {
        taken_bind_group_layouts,
        bind_group_layouts,
        push_constant_layout: maybe_push_constant_layout.clone(),
    };

    // Create the pipeline layout and return it
    Ok(internal_create_pipeline_layout(graphics, shader, names))
}

// Internal function that will take a reflected shader and create a pipeline layout for it
fn internal_create_pipeline_layout(
    graphics: &Graphics,
    shader: ReflectedShader,
    names: &[&str],
) -> (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>) {
    // Before creating the layout, check if we already have a corresponding one in cache
    if let Some(cached) = graphics.0.cached.pipeline_layouts.get(&shader) {
        log::debug!("Found pipeline layout in cache for {names:?}, using it...");
        return (Arc::new(shader), cached.value().clone());
    } else {
        log::warn!("Did not find cached pipeline layout for {names:?}");
    }

    // Fetch (and cache if necessary) the empty bind group layout
    let cached = &graphics.0.cached;
    let empty_bind_group_layout = cached
        .bind_group_layouts
        .entry(BindGroupLayout {
            bind_entry_layouts: Vec::new(),
        })
        .or_insert_with(|| {
            // Create the BindGroupLayoutDescriptor for the BindGroupEntries
            let descriptor = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[],
            };

            // Create the bind group layout and add it to the cache
            Arc::new(graphics.device().create_bind_group_layout(&descriptor))
        })
        .clone();

    // Create the empty bind group
    cached.bind_groups.entry(Vec::new()).or_insert_with(|| {
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &empty_bind_group_layout,
            entries: &[],
        };

        Arc::new(graphics.device().create_bind_group(&desc))
    });

    // Add the uncached bind group entries to the graphics cache
    for (bind_group_index, bind_group_layout) in shader.bind_group_layouts.iter().enumerate() {
        // If the bind group is hopped over, always use the default hop bind group layout
        let Some(bind_group_layout) = bind_group_layout else {
            continue;
        };

        // Add the bind group to the cache if it's missing
        if !cached.bind_group_layouts.contains_key(bind_group_layout) {
            log::warn!(
                "Did not find cached bind group layout for set = {bind_group_index}, in {names:?}"
            );

            // Convert each entry from this group to a WGPU BindGroupLayoutEntry
            let entries = bind_group_layout
                .bind_entry_layouts
                .iter()
                .map(|value| wgpu::BindGroupLayoutEntry {
                    binding: value.binding,
                    visibility: visibility_to_wgpu_stage(&value.visibility),
                    ty: map_binding_type(value),
                    count: value.count,
                })
                .collect::<Vec<_>>();

            // Create the BindGroupLayoutDescriptor for the BindGroupEntries
            let descriptor = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            };

            log::trace!("create bind group layout entries: {:?}", entries);

            // Create the bind group layout and add it to the cache
            let layout = graphics.device().create_bind_group_layout(&descriptor);
            let layout = Arc::new(layout);
            cached
                .bind_group_layouts
                .insert(bind_group_layout.clone(), layout);
        }
    }

    // Fetch the bind group layouts from the cache
    let bind_group_layouts = shader
        .bind_group_layouts
        .iter()
        .map(|bind_group_layout| {
            bind_group_layout
                .as_ref()
                .map(|bind_group_layout| cached.bind_group_layouts.get(&bind_group_layout).unwrap())
        })
        .collect::<Vec<_>>();

    // Convert the bind group layouts hash map references to proper references (use hop bind group layout)
    let bind_group_layouts = bind_group_layouts
        .iter()
        .map(|x| {
            x.as_ref()
                .map(|x| &***x)
                .unwrap_or(&*empty_bind_group_layout)
        })
        .take(shader.taken_bind_group_layouts)
        .collect::<Vec<_>>();

    // Convert the custom push constant range to wgpu push constant ranges
    let push_constant_ranges = if let Some(range) = shader.push_constant_layout {
        match range {
            PushConstantLayout::SplitVertexFragment {
                vertex: vertex_size,
                fragment: fragment_size,
            } => vec![
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..(vertex_size.get()),
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: vertex_size.get()..(fragment_size.get() + vertex_size.get()),
                },
            ],

            PushConstantLayout::Single(size, visibility) => {
                vec![wgpu::PushConstantRange {
                    stages: super::visibility_to_wgpu_stage(&visibility),
                    range: 0..size.get(),
                }]
            }
        }
    } else {
        Vec::default()
    };

    // Some logging now
    log::debug!("Using {} bind group layout(s)", bind_group_layouts.len());
    log::debug!(
        "Using {} push constants range(s)",
        push_constant_ranges.len()
    );

    // Create the pipeline layout
    let layout = graphics
        .device()
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &push_constant_ranges,
        });

    // Put it inside the graphics cache
    let layout = Arc::new(layout);
    graphics
        .0
        .cached
        .pipeline_layouts
        .insert(shader.clone(), layout.clone());
    log::debug!("Saved pipeline layout for {names:?} in graphics cache");

    (Arc::new(shader), layout)
}

// Reflects a uniform buffer using user's shader settings
fn reflect_uniform_buffer(
    resource: &BindResourceType,
    _type: &spirq::ty::Type,
) -> Result<BindResourceType, BufferValidationError> {
    // Make sure the resource is a uniform buffer
    let BindResourceType::UniformBuffer { size: compiler } = resource else {
        return Err(BufferValidationError::NotUniformBuffer)  
    };

    // Get the size of the type
    let shader = get_spirq_type_size(_type).unwrap();

    /*
    // Make sure the sizes are multiples
    if *compiler % shader != 0  {
        return Err(BufferValidationError::MismatchSize {
            compiler: *compiler,
            shader,
        });
    }
    */

    Ok(resource.clone())
}

// Reflects a storage buffer using user's shader settings
fn reflect_storage_buffer(
    resource: &BindResourceType,
    shader_access: &spirq::AccessType,
    _type: &spirq::ty::Type,
) -> Result<BindResourceType, BufferValidationError> {
    // Make sure the resource is a storage buffer
    let BindResourceType::StorageBuffer { size: compiler_size, access: compiler_access } = resource else {
        return Err(BufferValidationError::NotUniformBuffer)  
    };

    // Get the size of the type
    let shader_size = get_spirq_type_size(_type).unwrap();
    //dbg!(shader_size);

    // Make sure the sizes are multiples
    /*
    if *compiler_size % shader_size != 0 {
        return Err(BufferValidationError::MismatchSize {
            compiler: *compiler_size,
            shader: shader_size,
        });
    }
    */

    // Make sure the accesses match up
    // TODO: Fix this, seems broken. Spirq bug?
    /*
    if shader_access != compiler_access {
        return Err(BufferValidationError::MismatchAccess {
            compiler: *compiler_access,
            shader: *shader_access
        });
    }
    */

    Ok(resource.clone())
}

// Reflects a sampler using user's shader settings
fn reflect_sampler(
    resource: &BindResourceType,
) -> Result<BindResourceType, SamplerValidationError> {
    // Make sure the resource is a sampler
    let BindResourceType::Sampler { .. } = resource else {
        return Err(SamplerValidationError::NotSampler)  
    };

    Ok(resource.clone())
}

// Reflects a storage texture using user's shader settings
fn reflect_storage_texture(
    name: &str,
    graphics: &Graphics,
    resource: &BindResourceType,
    shader_access: &spirq::AccessType,
    _type: &spirq::ty::StorageImageType,
) -> Result<BindResourceType, TextureValidationError> {
    // Make sure the resource is a storage texture
    let BindResourceType::StorageTexture { 
        access: compiler_access,
        format,
        view_dimension
    } = resource else {
        return Err(TextureValidationError::NotSampledTexture)  
    };

    // Make sure the view dimensions match up
    if *view_dimension != map_spirv_dim(_type.dim, _type.is_array) {
        return Err(TextureValidationError::MismatchViewDimension {
            compiler: *view_dimension,
            shader: map_spirv_dim(_type.dim, _type.is_array),
        });
    }

    // Make sure the format matches up
    if *format != map_spirv_format(_type.fmt) {
        return Err(TextureValidationError::MismatchFormat {
            compiler: *format,
            shader: map_spirv_format(_type.fmt),
        });
    }

    // Make sure the accesses match up
    // TODO: Fix this, seems broken. Spirq bug?
    /*
    if shader_access != compiler_access {
        return Err(TextureValidationError::MismatchAccess {
            compiler: *compiler_access,
            shader: *shader_access
        });
    }
    */

    Ok(resource.clone())
}

// Reflects a sampled texture using user's shader settings
fn reflect_sampled_texture(
    name: &str,
    graphics: &Graphics,
    resource: &BindResourceType,
    _type: &spirq::ty::SampledImageType,
) -> Result<BindResourceType, TextureValidationError> {
    // Make sure the resource is a sampled texture
    let BindResourceType::SampledTexture {
        sample_type,
        sampler_binding,
        view_dimension,
        format,
    } = resource else {
        return Err(TextureValidationError::NotSampledTexture)  
    };

    // Make sure the view dimensions match up
    if *view_dimension != map_spirv_dim(_type.dim, _type.is_array) {
        return Err(TextureValidationError::MismatchViewDimension {
            compiler: *view_dimension,
            shader: map_spirv_dim(_type.dim, _type.is_array),
        });
    }

    // Make sure the sample type matches up
    if *sample_type != map_spirv_scalar_type(_type.scalar_ty.clone(), *format) {
        return Err(TextureValidationError::MismatchSampleType {
            compiler: *sample_type,
            shader: map_spirv_scalar_type(_type.scalar_ty.clone(), *format),
        });
    }

    Ok(resource.clone())
}
