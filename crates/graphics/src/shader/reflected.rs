use std::{hash::Hash, sync::Arc};

use crate::{
    visibility_to_wgpu_stage, Compiled, ComputeModule,
    FragmentModule, Graphics, ModuleKind, ModuleVisibility,
    ShaderModule, TexelChannels, VertexModule, ShaderReflectionError,
};
use ahash::{AHashMap, AHashSet};
use arrayvec::ArrayVec;
use naga::{AddressSpace, TypeInner};
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
    SharedVertexFragment(u32),
    VertexFragment { vertex: u32, fragment: u32 },
    Compute(u32),
}

impl PushConstantLayout {
    // Create a new PushConstantLayout from a visibility and a size
    pub fn new(visibility: ModuleVisibility, size: u32) -> Self {
        match visibility {
            ModuleVisibility::Vertex => Self::VertexFragment {
                vertex: size,
                fragment: 0,
            },
            ModuleVisibility::Fragment => Self::VertexFragment {
                vertex: 0,
                fragment: size,
            },
            ModuleVisibility::VertexFragment => {
                Self::SharedVertexFragment(size)
            }
            ModuleVisibility::Compute => Self::Compute(size),
        }
    }

    // Insert (combine) another PushConstantLayout into self
    pub fn insert(&mut self, other: Self) {
        match (self, other) {
            (
                PushConstantLayout::SharedVertexFragment(size_a),
                PushConstantLayout::SharedVertexFragment(size_b),
            ) if *size_a == size_b => {}
            (
                PushConstantLayout::VertexFragment {
                    vertex: vertex_a,
                    fragment: fragment_a,
                },
                PushConstantLayout::VertexFragment {
                    vertex: vertex_b,
                    fragment: fragment_b,
                },
            ) => {
                if (*vertex_a == 0) == (vertex_b == 0) {
                    panic!()
                }

                *vertex_a += vertex_b;
                *fragment_a += fragment_b;
            }
            (
                PushConstantLayout::Compute(_),
                PushConstantLayout::Compute(_),
            ) => {}
            _ => panic!(),
        }
    }
}

// The type of BindingEntry. This is fetched from the given
// For now, only buffers, samplers, and texture are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindResourceType {
    // Either a Uniform buffer or a Storage buffer
    Buffer {
        size: usize,
        storage: bool,
        read: bool,
        write: bool,
    },

    // A sampler type that we can use to sample textures (sampler2D)
    Sampler {
        format: wgpu::TextureFormat,
        sampler_binding: wgpu::SamplerBindingType,
    },

    // A texture type without a sampler (texture2D)
    Texture {
        format: wgpu::TextureFormat,
        sample_type: wgpu::TextureSampleType,
        sampler_binding: wgpu::SamplerBindingType,
        view_dimension: wgpu::TextureViewDimension,
    },
}

// Internal data that is passed to module creation
// This is used by the compiler to define formats and types used by shaders
struct InternalDefinitions<'a> {
    texture_formats: &'a super::TextureFormats,
    texture_dimensions: &'a super::TextureDimensions,
    uniform_buffer_pod_types: &'a super::UniformBufferPodTypes,
    push_constant_ranges: &'a super::PushConstantRanges,
}

// Convert a reflected bind entry layout to a wgpu binding type
fn map_binding_type(value: &BindResourceLayout) -> wgpu::BindingType {
    match value.resource_type {
        BindResourceType::Buffer {
            size,
            storage,
            read,
            write,
        } => wgpu::BindingType::Buffer {
            ty: match storage {
                true => wgpu::BufferBindingType::Storage {
                    read_only: read && !write,
                },
                false => wgpu::BufferBindingType::Uniform,
            },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindResourceType::Sampler {
            sampler_binding, ..
        } => wgpu::BindingType::Sampler(sampler_binding),
        BindResourceType::Texture {
            sample_type,
            view_dimension,
            ..
        } => wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled: false,
        },
    }
}

// Create a pipeline layout for a combination of shader modules using the specified definitions
pub(super) fn create_pipeline_layout(
    graphics: &Graphics,
    names: &[&str],
    modules: &[&naga::Module],
    visibility: &[ModuleVisibility],
    texture_formats: &super::TextureFormats,
    texture_dimensions: &super::TextureDimensions,
    uniform_buffer_pod_types: &super::UniformBufferPodTypes,
    push_constant_range: &super::PushConstantRanges,
) -> Result<(Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>), ShaderReflectionError> {
    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: [Option<AHashMap<u32, BindResourceLayout>>; 4] =
        [None, None, None, None];

    // Return error if the user defined a push constant that is greater than the device size
    

    // TODO: Implement this

    // Ease of use
    let definitions = InternalDefinitions {
        texture_formats,
        texture_dimensions,
        uniform_buffer_pod_types,
        push_constant_ranges: push_constant_range,
    };

    // Iterate through the sets and create the bind groups for each of the resources
    for (index, module) in modules.iter().enumerate() {
        let visibility = visibility[index];
        for set in 0..4 {
            let types = &module.types;
            let vars = &module.global_variables;

            // Iterate over the global variables and get their binding entry
            let iter = vars
                .iter()
                .filter(|(_, value)| {
                    value.binding.is_some() && value.name.is_some()
                })
                .map(|(_, x)| x)
                .filter(|value| {
                    matches!(value.space, AddressSpace::Uniform)
                        | matches!(
                            value.space,
                            AddressSpace::Storage { .. }
                        )
                        | matches!(value.space, AddressSpace::Handle)
                })
                .filter(|value| types.get_handle(value.ty).is_ok());

            for value in iter {
                let name = &value.name.as_ref().unwrap();
                let binding = &value.binding.as_ref().unwrap();
                let set = binding.group;
                let binding = binding.binding;
                let type_ = types.get_handle(value.ty).unwrap();
                let inner = &type_.inner;

                // Get the merged group layout and merged group entry layouts
                let merged_group_layout = &mut groups[set as usize];
                let merged_group_entry_layouts = merged_group_layout
                    .get_or_insert_with(|| Default::default());

                // Get the binding type for this global variable
                let binding_type = match inner {
                    // Uniform Buffers
                    TypeInner::Struct { span: size, .. } => {
                        // TODO: VALIDATE BUFFER (make sure it's same size and alignment)
                        Some(reflect_buffer(
                            &name,
                            graphics,
                            &definitions,
                        ))
                    }

                    // Uniform Textures
                    TypeInner::Image {
                        dim,
                        class,
                        arrayed,
                    } => {
                        // TODO: VALIDATE TEXTURE (make sure it's same dimension, type, and texel type)
                        Some(reflect_texture(
                            &name,
                            class,
                            dim,
                            graphics,
                            &definitions,
                            *arrayed,
                        ))
                    }

                    // Uniform Sampler
                    TypeInner::Sampler { comparison } => {
                        // TODO: VALIDATE SAMPLEr (make sure it's same texel type, type)
                        Some(reflect_sampler(
                            &name,
                            graphics,
                            &definitions,
                            *comparison,
                        ))
                    }

                    _ => None,
                };

                // If none, ignore
                let Some(resource_type) = binding_type else {
                    continue
                };

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
                        merged.visibility.insert(old.visibility);
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
        push_constant_layout: push_constant_range.clone(),
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
        log::debug!("Found pipeline layout in cache, using it...");
        return (Arc::new(shader), cached.value().clone());
    } else {
        log::warn!(
            "Did not find cached pipeline layout for {names:?}"
        );
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
            Arc::new(
                graphics
                    .device()
                    .create_bind_group_layout(&descriptor),
            )
        });

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

    // Fetch the bind group layouts from the cache
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
                .unwrap_or(&**empty_bind_group_layout)
        })
        .take(shader.last_valid_bind_group_layout + 1)
        .collect::<Vec<_>>();

    // Convert the custom push constant range to wgpu push constant ranges
    let mut push_constant_ranges = if let Some(range) =
        shader.push_constant_layout
    {
        match range {
            PushConstantLayout::SharedVertexFragment(size) => {
                vec![wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: 0..size,
                }]
            }
            PushConstantLayout::VertexFragment {
                vertex: vertex_size,
                fragment: fragment_size,
            } => vec![
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..vertex_size,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: vertex_size..(fragment_size + vertex_size),
                },
            ],
            PushConstantLayout::Compute(size) => {
                vec![wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..size,
                }]
            }
        }
    } else {
        Vec::default()
    };
    push_constant_ranges
        .retain(|x| (x.range.end - x.range.start) > 0);

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

    (Arc::new(shader), layout)
}

fn reflect_buffer(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> BindResourceType {
    // TODO: Implement storage buffers
    let pod_info =
        definitions.uniform_buffer_pod_types.get(name).unwrap();
    let size = pod_info.size();

    BindResourceType::Buffer {
        size,
        storage: false,
        read: true,
        write: false,
    }
}

fn reflect_sampler(
    name: &str,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
    comparison: bool,
) -> BindResourceType {
    BindResourceType::Sampler {
        sampler_binding: wgpu::SamplerBindingType::Filtering,
        format: definitions
            .texture_formats
            .get(name)
            .unwrap()
            .format(),
    }
}

fn reflect_texture(
    name: &str,
    class: &naga::ImageClass,
    dim: &naga::ImageDimension,
    graphics: &Graphics,
    definitions: &InternalDefinitions,
    arrayed: bool,
) -> BindResourceType {
    BindResourceType::Texture {
        sample_type: match class {
            naga::ImageClass::Sampled { kind, multi } => match kind {
                naga::ScalarKind::Sint => {
                    wgpu::TextureSampleType::Sint
                }
                naga::ScalarKind::Uint => {
                    wgpu::TextureSampleType::Uint
                }
                naga::ScalarKind::Float => {
                    let adapter = graphics.adapter();
                    let info = definitions
                        .texture_formats
                        .get(name)
                        .unwrap();
                    let format = info.format();
                    let flags = adapter
                        .get_texture_format_features(format)
                        .flags;

                    let depth = matches!(
                        info.channels(),
                        TexelChannels::Depth
                    );

                    if flags.contains(
                        TextureFormatFeatureFlags::FILTERABLE,
                    ) && !depth
                    {
                        wgpu::TextureSampleType::Float {
                            filterable: true,
                        }
                    } else {
                        wgpu::TextureSampleType::Float {
                            filterable: false,
                        }
                    }
                }
                _ => panic!(),
            },
            naga::ImageClass::Depth { multi } => todo!(),
            naga::ImageClass::Storage { format, access } => todo!(),
        },

        view_dimension: match dim {
            naga::ImageDimension::D1 => {
                wgpu::TextureViewDimension::D1
            }
            naga::ImageDimension::D2 => {
                wgpu::TextureViewDimension::D2
            }
            naga::ImageDimension::D3 => {
                wgpu::TextureViewDimension::D3
            }
            naga::ImageDimension::Cube => {
                wgpu::TextureViewDimension::Cube
            }
        },
        format: {
            let format =
                definitions.texture_formats.get(name).unwrap();
            format.format()
        },
        sampler_binding: {
            let adapter = graphics.adapter();
            let info = definitions.texture_formats.get(name).unwrap();
            let format = info.format();
            let flags =
                adapter.get_texture_format_features(format).flags;

            let depth =
                matches!(info.channels(), TexelChannels::Depth);

            if flags.contains(TextureFormatFeatureFlags::FILTERABLE)
                && !depth
            {
                wgpu::SamplerBindingType::Filtering
            } else {
                wgpu::SamplerBindingType::NonFiltering
            }
        },
    }
}
