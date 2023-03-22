use std::{hash::Hash, num::NonZeroU32, sync::Arc};

use crate::{
    visibility_to_wgpu_stage, BufferValidationError, Compiled,
    ComputeModule, ElementType, FragmentModule, Graphics, ModuleKind,
    ModuleVisibility, PushConstantValidationError,
    SamplerValidationError, ShaderModule, ShaderReflectionError,
    TexelChannels, TexelInfo, TextureValidationError, VertexModule,
};
use ahash::{AHashMap, AHashSet};
use arrayvec::ArrayVec;
use spirq::DescriptorType;
use utils::enable_in_range;
use wgpu::TextureFormatFeatureFlags;

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedShader {
    pub last_valid_bind_group_layout: usize,
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
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
    pub fn single(
        size: usize,
        visibility: ModuleVisibility,
    ) -> Option<Self> {
        let size = NonZeroU32::new(size as u32)?;
        Some(Self::Single(size, visibility))
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
            PushConstantLayout::SplitVertexFragment { .. } => {
                ModuleVisibility::VertexFragment
            }
        }
    }

    // Get the MAX size required to set the push constant bytes
    pub fn size(&self) -> NonZeroU32 {
        match self {
            PushConstantLayout::Single(x, _) => *x,
            PushConstantLayout::SplitVertexFragment {
                vertex,
                fragment,
            } => NonZeroU32::new(vertex.get() + fragment.get())
                .unwrap(),
        }
    }
}

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
        read: bool,
        write: bool,
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
        access: wgpu::StorageTextureAccess,
        format: wgpu::TextureFormat,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
    },
}

// Internal data that is passed to module creation
// This is used by the compiler to define formats and types used by shaders
struct InternalDefinitions<'a> {
    resource_binding_types: &'a super::ResourceBindingTypes,
    maybe_push_constant_layout: &'a super::MaybePushConstantLayout,
}

// Convert a reflected bind entry layout to a wgpu binding type
pub(super) fn map_binding_type(
    value: &BindResourceLayout,
) -> wgpu::BindingType {
    match value.resource_type {
        BindResourceType::UniformBuffer { size } => {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        }

        BindResourceType::StorageBuffer { size, read, write } => {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: read && !write,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        }

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
            sample_type,
            access,
            view_dimension,
        } => wgpu::BindingType::StorageTexture {
            access,
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
            let flags =
                adapter.get_texture_format_features(format).flags;

            let depth =
                matches!(info.channels(), TexelChannels::Depth);

            if flags.contains(TextureFormatFeatureFlags::FILTERABLE)
                && !depth
            {
                wgpu::TextureSampleType::Float { filterable: true }
            } else {
                wgpu::TextureSampleType::Float { filterable: false }
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

    if flags.contains(TextureFormatFeatureFlags::FILTERABLE) && !depth
    {
        wgpu::SamplerBindingType::Filtering
    } else {
        wgpu::SamplerBindingType::NonFiltering
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
) -> Result<
    (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>),
    ShaderReflectionError,
> {
    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: [Option<AHashMap<u32, BindResourceLayout>>; 4] =
        [None, None, None, None];

    // Return error if the user defined a push constant that is greater than the device size
    // or if there isn't push constants for the specified module in the shaders
    if let Some(push_constant_layout) = maybe_push_constant_layout {
        // We always assume that the "visibility" array given is either [Vertex, Fragment] or [Compute]
        // Under that assumption, all we really need to check is the compute module and if it contains the proper push constant
        let compute =
            matches!(visibility[0], ModuleVisibility::Compute);
        match (push_constant_layout.visibility(), compute) {
            (ModuleVisibility::Vertex, false) => {},
            (ModuleVisibility::Fragment, false) => {},
            (ModuleVisibility::VertexFragment, false) => {},
            (ModuleVisibility::Compute, true) => {},                
            _ => return Err(ShaderReflectionError::PushConstantValidation(
                PushConstantValidationError::PushConstantVisibilityIntersect
            )),
        }

        // Check size and make sure it's valid
        let defined = push_constant_layout.size().get();
        let max = graphics.device().limits().max_push_constant_size;
        if defined > max {
            return Err(ShaderReflectionError::PushConstantValidation(PushConstantValidationError::PushConstantSizeTooBig(
                defined, max
            )));
        }

        // Get the push constant sizes as defined in the shader modules
        let mut iter = modules
            .iter()
            .flat_map(|module| module
                .vars
                .iter()
                .filter_map(|x| match x {
                    spirq::Variable::PushConstant { name, ty } => ty.nbyte().map(|x| x as u32),
                    _ => None
                })
            );
        let (first, second) = (iter.next(), iter.next());

        // Validate the push constants and check if they were defined properly in the shader
        match push_constant_layout {
            PushConstantLayout::Single(size, _) => {
                if second.is_none() {
                    size.get() == first.unwrap()
                } else {
                    todo!()
                }
            }
            PushConstantLayout::SplitVertexFragment {
                vertex,
                fragment,
            } => {
                if first.is_some() && second.is_some() {
                    vertex.get() == first.unwrap()
                        && fragment.get() == second.unwrap()
                } else {
                    todo!()
                }
            }
        };
    }

    // Ease of use
    let definitions = InternalDefinitions {
        resource_binding_types,
        maybe_push_constant_layout,
    };

    // Iterate through the sets and create the bind groups for each of the resources
    for (index, module) in modules.iter().enumerate() {
        let visibility = visibility[index];

        // Create a hashmap that contains all the variables per group
        let mut sets = AHashMap::<u32, Vec<spirq::Variable>>::default();

        let iter = module
            .vars
            .iter()
            .filter_map(|variable| if let spirq::Variable::Descriptor { desc_bind, .. } = variable {
                Some((desc_bind.set(), variable))
            } else {
                None
            });


        // Add the binded variables to the hashmap
        for (set, variable) in iter {
            let vec = sets.entry(set).or_default();
            vec.push(variable.clone());
        }
        
        for set in 0..4 {
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
                let merged_group_entry_layouts = merged_group_layout
                    .get_or_insert_with(|| Default::default());

                // Get the binding type for this global variable
                let binding_type = match desc_ty {
                    DescriptorType::Sampler() => {
                        Some(reflect_sampler(&name, graphics, &definitions)
                            .map_err(ShaderReflectionError::SamplerValidation))
                    },
                    
                    DescriptorType::SampledImage() => {
                        Some(reflect_sampled_texture(&name, graphics, &definitions)
                            .map_err(ShaderReflectionError::TextureValidation))
                    },
                    
                    DescriptorType::StorageImage(access) => {
                        Some(reflect_storage_texture(&name, graphics, &definitions)
                            .map_err(ShaderReflectionError::TextureValidation))
                    },
                    
                    DescriptorType::UniformBuffer() => {
                        Some(reflect_uniform_buffer(&name, graphics, &definitions)
                            .map_err(ShaderReflectionError::BufferValidation))
                    },

                    DescriptorType::StorageBuffer(access) => {
                        Some(reflect_storage_buffer(&name, graphics, &definitions)
                            .map_err(ShaderReflectionError::BufferValidation))
                    },

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
                };

                // Merge each entry for this group individually
                match merged_group_entry_layouts.entry(binding) {
                    // Merge an already existing layout with the new one
                    std::collections::hash_map::Entry::Occupied(
                        mut occupied,
                    ) => {
                        let merged_bind_entry_layout =
                            occupied.get_mut();
                        let old = bind_entry_layout;
                        let merged = merged_bind_entry_layout;

                        // Make sure the currently merged layout and the new layout
                        // have well... the same layout
                        if old.resource_type != merged.resource_type {
                            panic!("Not the same layout");
                        }

                        // Merge the visibility to allow more modules to access this entry
                        merged.visibility.try_insert(old.visibility).unwrap();
                    }

                    // If the spot is vacant, add the bind entry layout for the first time
                    std::collections::hash_map::Entry::Vacant(
                        vacant,
                    ) => {
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
        .collect::<ArrayVec<Option<BindGroupLayout>, 4>>();

    // Calculate the last valid bind group layout before we see the first None (starting from the back)
    let last_valid_bind_group_layout = bind_group_layouts
        .iter()
        .rposition(|x| x.is_some())
        .unwrap_or_default();

    // Create a reflected shader with the given compiler params
    let shader = ReflectedShader {
        last_valid_bind_group_layout,
        bind_group_layouts: bind_group_layouts.into_inner().unwrap(),
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
    if let Some(cached) =
        graphics.0.cached.pipeline_layouts.get(&shader)
    {
        log::debug!("Found pipeline layout in cache for {names:?}, using it...");
        return (Arc::new(shader), cached.value().clone());
    } else {
        log::warn!("Did not find cached pipeline layout for {names:?}");
    }

    log::trace!("internal_create_pipeline_layout: Start");

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
            Arc::new(
                graphics
                    .device()
                    .create_bind_group_layout(&descriptor),
            )
        })
        .clone();

    // Create the empty bind group
    log::trace!("internal_create_pipeline_layout: Empty Bind Group Start");
    cached.bind_groups.entry(Vec::new()).or_insert_with(|| {
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &empty_bind_group_layout,
            entries: &[],
        };

        Arc::new(graphics.device().create_bind_group(&desc))
    });
    log::trace!("internal_create_pipeline_layout: Empty Bind Group End");

    // Add the uncached bind group entries to the graphics cache
    log::trace!("internal_create_pipeline_layout: Bind Groups Start");
    for (bind_group_index, bind_group_layout) in
        shader.bind_group_layouts.iter().enumerate()
    {
        // If the bind group is hopped over, always use the default hop bind group layout
        let Some(bind_group_layout) = bind_group_layout else {
            continue;
        };

        // Add the bind group to the cache if it's missing
        if !cached.bind_group_layouts.contains_key(bind_group_layout)
        {
            log::warn!("Did not find cached bind group layout for set = {bind_group_index}, in {names:?}");

            // TODO: Validate the bindings and groups
            // Convert each entry from this group to a WGPU BindGroupLayoutEntry
            let entries = bind_group_layout
                .bind_entry_layouts
                .iter()
                .map(|value| wgpu::BindGroupLayoutEntry {
                    binding: value.binding,
                    visibility: visibility_to_wgpu_stage(
                        &value.visibility,
                    ),
                    ty: map_binding_type(value),
                    count: None,
                })
                .collect::<Vec<_>>();

            // Create the BindGroupLayoutDescriptor for the BindGroupEntries
            let descriptor = wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            };

            // Create the bind group layout and add it to the cache
            let layout = graphics
                .device()
                .create_bind_group_layout(&descriptor);
            let layout = Arc::new(layout);
            cached
                .bind_group_layouts
                .insert(bind_group_layout.clone(), layout);
        }
    }
    log::trace!("internal_create_pipeline_layout: Bind Groups End");

    // Fetch the bind group layouts from the cache
    log::trace!("internal_create_pipeline_layout: Bind Group Layouts Start");
    let bind_group_layouts = shader
        .bind_group_layouts
        .iter()
        .map(|bind_group_layout| {
            bind_group_layout.as_ref().map(|bind_group_layout| {
                cached
                    .bind_group_layouts
                    .get(&bind_group_layout)
                    .unwrap()
            })
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
        .take(shader.last_valid_bind_group_layout + 1)
        .collect::<Vec<_>>();
    log::trace!("internal_create_pipeline_layout: Bind Group Layouts End");

    // Convert the custom push constant range to wgpu push constant ranges
    let push_constant_ranges = if let Some(range) =
        shader.push_constant_layout
    {
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
                    range: vertex_size.get()
                        ..(fragment_size.get() + vertex_size.get()),
                },
            ],

            PushConstantLayout::Single(size, visibility) => {
                vec![wgpu::PushConstantRange {
                    stages: super::visibility_to_wgpu_stage(
                        &visibility,
                    ),
                    range: 0..size.get(),
                }]
            }
        }
    } else {
        Vec::default()
    };

    // Create the pipeline layout
    let layout = graphics.device().create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &push_constant_ranges,
        },
    );

    // Put it inside the graphics cache
    let layout = Arc::new(layout);
    graphics
        .0
        .cached
        .pipeline_layouts
        .insert(shader.clone(), layout.clone());
    log::debug!(
        "Saved pipeline layout for {names:?} in graphics cache"
    );
        
    log::trace!("internal_create_pipeline_layout: End");

    (Arc::new(shader), layout)
}

fn reflect_uniform_buffer(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> Result<BindResourceType, BufferValidationError> {
    // TODO: VALIDATE BUFFER (make sure it's same size, access)
    let binding = definitions
        .resource_binding_types
        .get(name)
        .unwrap()
        .clone();
    Ok(binding)
}

fn reflect_storage_buffer(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> Result<BindResourceType, BufferValidationError> {
    // TODO: VALIDATE BUFFER (make sure it's same size, access)
    let binding = definitions
        .resource_binding_types
        .get(name)
        .unwrap()
        .clone();
    Ok(binding)
}

fn reflect_sampler(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> Result<BindResourceType, SamplerValidationError> {
    // TODO: VALIDATE SAMPLER (make sure it's same texel type, type)
    let binding = definitions
        .resource_binding_types
        .get(name)
        .unwrap()
        .clone();
    Ok(binding)
}

fn reflect_storage_texture(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> Result<BindResourceType, TextureValidationError> {
    // TODO: VALIDATE TEXTURE (make sure it's same dimension, type, and texel type)
    let binding = definitions
        .resource_binding_types
        .get(name)
        .unwrap()
        .clone();
    Ok(binding)
}

fn reflect_sampled_texture(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> Result<BindResourceType, TextureValidationError> {
    // TODO: VALIDATE TEXTURE (make sure it's same dimension, type, and texel type)
    let binding = definitions
        .resource_binding_types
        .get(name)
        .unwrap()
        .clone();
    Ok(binding)
}
