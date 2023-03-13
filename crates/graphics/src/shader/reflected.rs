use std::{hash::Hash, sync::Arc};

use crate::{
    Compiled, FragmentModule, Graphics, ModuleKind, ShaderModule,
    TexelChannels, VertexModule, ModuleVisibility, visibility_to_wgpu_stage,
};
use ahash::{AHashMap, AHashSet};
use naga::{AddressSpace, ResourceBinding, TypeInner};
use wgpu::TextureFormatFeatureFlags;

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedShader {
    pub last_valid_bind_group_layout: usize,
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
    pub push_constant_ranges: Vec<PushConstantRange>,
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
    pub visiblity: ModuleVisibility,
}

// Push constant uniform data that we will fill field by field
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantRange {
    pub visibility: ModuleVisibility,
    pub start: usize,
    pub end: usize,
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
fn map_binding_type(
    value: &BindResourceLayout,
) -> wgpu::BindingType {
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
    definitions: &InternalDefinitions,
    names: &[&str],
) -> (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>) {
    
}

fn fun_name(graphics: &Graphics, shader: &ReflectedShader) -> (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>) {
    // Before creating the layout, check if we already have a corresponding one in cache
    if let Some(cached) =
        graphics.0.cached.pipeline_layouts.get(&shader)
    {
        log::debug!("Found pipeline layout in cache, using it...");
        return cached.clone();
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
        if !cached.bind_group_layouts.contains_key(bind_group_layout) {
            log::warn!("Did not find cached bind group layout for set = {bind_group_index}, in {names:?}");

            // TODO: Validate the bindings and groups
            // Convert each entry from this group to a WGPU BindGroupLayoutEntry
            let entries = bind_group_layout
                .bind_entry_layouts
                .iter()
                .map(|value| wgpu::BindGroupLayoutEntry {
                    binding: value.binding,
                    visibility: visibility_to_wgpu_stage(&value.visiblity),
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

    // Convert the custom push constant ranges to wgpu push constant ranges
    let push_constant_ranges = shader
        .push_constant_ranges
        .iter()
        .map(|layout| wgpu::PushConstantRange {
            stages: visibility_to_wgpu_stage(&layout.visibility),
            range: (layout.start as u32)..(layout.end as u32),
        })
        .collect::<Vec<_>>();

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
    let pod_info = definitions.uniform_buffer_pod_types.get(name).unwrap();
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
    graphics: &Graphics,
    definitions: &InternalDefinitions,
) -> BindResourceType {
    todo!()
    /*
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
    */
}
