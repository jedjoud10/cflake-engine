use std::{
    collections::hash_map::DefaultHasher, hash::Hash, sync::Arc,
};

use crate::{Graphics, ShaderModule};
use ahash::{AHashMap, AHashSet};
use arrayvec::ArrayVec;
use itertools::Itertools;
use naga::{AddressSpace, ResourceBinding, TypeInner};

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedShader {
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
}

// This container stores all data related to reflected modules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedModule {
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
}

// A bind group contains one or more bind entries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupLayout {
    pub bind_entry_layouts: Vec<BindEntryLayout>,
}

// A binding entry is a single binding resource from within a group
// Eg. a uniform buffer, a sampler, a texture, or storage buffer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindEntryLayout {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub binding_type: BindingType,
    pub visiblity: wgpu::ShaderStages,
}

// The type of BindingEntry.
// For now, only buffers, samplers, and texture are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingType {
    Buffer {
        buffer_binding: wgpu::BufferBindingType,
        members: Vec<StructMemberLayout>,
        size: u32,
    },
    Sampler {
        sampler_binding: wgpu::SamplerBindingType,
    },
    Texture {
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
    },
}

// Struct member type for fields of Buffer structures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructMemberLayout {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub struct_type: StructMemberType,
}

// Types of buffer structure fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructMemberType {
    Scalar {
        kind: naga::ScalarKind,
    },

    Vector {
        size: naga::VectorSize,
        kind: naga::ScalarKind,
    },

    Matrix {
        columns: naga::VectorSize,
        rows: naga::VectorSize,
    },
}

// Merge multiple reflected modules to create a reflected shader
pub fn merge_reflected_modules_to_shader(
    modules: &[&ReflectedModule],
) -> ReflectedShader {
    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: [Option<AHashMap<u32, BindEntryLayout>>; 4] =
        [None, None, None, None];

    // Merge differnet bind modules into one big hashmap
    for module in modules {
        for (group_index, bind_group_layout) in module.bind_group_layouts.iter().enumerate() {
            // Skip this bind group if it was hopped over in the shader
            let Some(bind_group_layout) = bind_group_layout else {
                continue;
            };

            // This bind group MUST contains at least ONE entry
            if bind_group_layout.bind_entry_layouts.len() == 0 {
                panic!("Bind group MUST contain at least ONE entry");
            }
            
            let merged_group_layout = &mut groups[group_index as usize];
            let merged_group_entry_layouts = merged_group_layout.get_or_insert_with(|| Default::default());

            // Merge each entry for this group individually
            for bind_entry_layout in bind_group_layout.bind_entry_layouts.iter() {                
                match merged_group_entry_layouts.entry(bind_entry_layout.binding) {
                    // Merge an already existing layout with the new one
                    std::collections::hash_map::Entry::Occupied(mut occupied) => {
                        let merged_bind_entry_layout = occupied.get_mut();
                        let old = bind_entry_layout;
                        let merged = merged_bind_entry_layout;

                        // Make sure the currently merged layout and the new layout
                        // have well... the same layout
                        if old.binding_type != merged.binding_type {
                            panic!("Not the same layout");
                        }

                        // Merge the visibility to allow more modules to access this entry
                        merged.visiblity.insert(old.visiblity);
                    },

                    // If the spot is vacant, add the bind entry layout for the first time
                    std::collections::hash_map::Entry::Vacant(vacant) => {
                        dbg!(bind_entry_layout);
                        vacant.insert(bind_entry_layout.clone());
                    },
                }
            }
        }
    }

    // Convert the entries back into bind groups
    let groups: [Option<BindGroupLayout>; 4] = groups
        .into_iter()
        .map(|entries| {
            entries.map(|entries| BindGroupLayout {
                bind_entry_layouts: entries.into_iter().map(|(_, x)| x).collect(),
            })
        })
        .collect::<Vec<_>>()
        .try_into().unwrap();
    ReflectedShader { bind_group_layouts: groups }
}

// Convert a given reflected shader to a pipeline layout (by creating it)
pub fn create_pipeline_layout_from_shader(
    graphics: &Graphics,
    shader: &ReflectedShader,
    names: &[&str],
) -> Arc<wgpu::PipelineLayout> {
    // Convert a reflected bind entry layout to a wgpu binding type
    fn map_binding_type(
        value: &BindEntryLayout,
    ) -> wgpu::BindingType {
        match value.binding_type {
            BindingType::Buffer { buffer_binding, .. } => {
                wgpu::BindingType::Buffer {
                    ty: buffer_binding,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            }
            BindingType::Sampler { sampler_binding } => {
                wgpu::BindingType::Sampler(sampler_binding)
            }
            BindingType::Texture {
                sample_type,
                view_dimension,
            } => wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled: false,
            },
        }
    }

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
    let empty_bind_group_layout = cached.bind_group_layouts.entry(BindGroupLayout {
        bind_entry_layouts: Vec::new(),
    }).or_insert_with(|| {
        // Create the BindGroupLayoutDescriptor for the BindGroupEntries
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        };

        // Create the bind group layout and add it to the cache
        Arc::new(graphics
            .device()
            .create_bind_group_layout(&descriptor))
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
    for (bind_group_index, bind_group_layout) in shader.bind_group_layouts.iter().enumerate() {
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
                    visibility: value.visiblity,
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
            cached.bind_group_layouts.insert(bind_group_layout.clone(), layout);
        }
    }

    // Fetch the bind group layouts from the cache
    let bind_group_layouts = shader
        .bind_group_layouts
        .iter()
        .map(|bind_group_layout| {
            bind_group_layout.as_ref().map(|bind_group_layout| {
                cached.bind_group_layouts.get(&bind_group_layout).unwrap()
            })
        })
        .collect::<Vec<_>>();

    // Convert the bind group layouts hash map references to proper references 
    let bind_group_layouts = bind_group_layouts
        .iter()
        .map(|x| x
            .as_ref()
            .map(|x| &***x)
            .unwrap_or(&**empty_bind_group_layout)
        )
        .collect::<Vec<_>>();

    // Create the pipeline layout
    let layout = graphics.device().create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
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
    layout
}

// Reflect a naga module's bindings and constants
pub fn reflect_module<M: ShaderModule>(
    naga: &naga::Module,
) -> ReflectedModule {
    let groups = reflect_binding_group::<M>(naga);
    ReflectedModule { bind_group_layouts: groups }
}

// Fetches the used binding groups of a given naga module
pub fn reflect_binding_group<M: ShaderModule>(
    naga: &naga::Module,
) -> [Option<BindGroupLayout>; 4] {
    let mut bind_group_layouts: [Option<BindGroupLayout>; 4] = [None, None, None, None];
    let entries = reflect_binding_entries::<M>(naga);

    // Merge the binding entries into their respective bind group layouts
    for bind_entry_layout in entries {
        let bind_group_layout = 
            &mut bind_group_layouts[bind_entry_layout.group as usize];
        let bind_group_layout = bind_group_layout.get_or_insert_with(|| BindGroupLayout {
            bind_entry_layouts: Vec::new(),
        });

        // Add the bind entry layout to the bind group layout
        bind_group_layout.bind_entry_layouts.push(bind_entry_layout);
    }

    bind_group_layouts
}

// Fetches the used binding entries of a given naga module
pub fn reflect_binding_entries<M: ShaderModule>(
    naga: &naga::Module,
) -> Vec<BindEntryLayout> {
    let types = &naga.types;
    let vars = &naga.global_variables;

    // Iterate over the global variables and get their binding entry
    vars.iter()
        .filter_map(|(_, value)| {
            value.binding.as_ref().map(|_| value)
        })
        .map(|value| {
            let ResourceBinding { group, binding } =
                *value.binding.as_ref().unwrap();

            // Get the type and address space of the variable
            let typed = types.get_handle(value.ty).unwrap();
            let type_inner = &typed.inner;
            let space = value.space;

            let binding_type = match &type_inner {
                // Uniform Buffers
                TypeInner::Struct {
                    members,
                    span: size,
                } => reflect_buffer(members, types, size, space),

                // Uniform Textures
                TypeInner::Image { dim, class, .. } => {
                    reflect_texture(class, dim)
                }

                // Uniform Sampler
                TypeInner::Sampler { comparison } => {
                    reflect_sampler(comparison)
                }
                _ => todo!(),
            };

            BindEntryLayout {
                name: value.name.clone().unwrap(),
                binding,
                group,
                binding_type,
                visiblity: naga_stage_to_wgpu_stage(&M::stage()),
            }
        })
        .collect::<Vec<_>>()
}

// Fetch the BindingType of a naga Struct (assuming it to be a buffer)
fn reflect_buffer(
    members: &Vec<naga::StructMember>,
    types: &naga::UniqueArena<naga::Type>,
    size: &u32,
    space: AddressSpace,
) -> BindingType {
    // Non UBO buffers not supported yet
    if space != AddressSpace::Uniform {
        panic!("Non UBO buffers not supported yet")
    }

    BindingType::Buffer {
        buffer_binding: wgpu::BufferBindingType::Uniform,
        members: members
            .iter()
            .map(|member| {
                let type_inner =
                    &types.get_handle(member.ty).unwrap().inner;
                let (size, struct_type) = match type_inner {
                    TypeInner::Scalar { kind, width } => (
                        *width as u32,
                        StructMemberType::Scalar { kind: *kind },
                    ),
                    TypeInner::Vector { size, kind, width } => {
                        let size2 =
                            *width as u32 * vector_size_to_u32(size);
                        (
                            size2,
                            StructMemberType::Vector {
                                size: *size,
                                kind: *kind,
                            },
                        )
                    }
                    TypeInner::Matrix {
                        columns,
                        rows,
                        width,
                    } => {
                        let size = *width as u32
                            * vector_size_to_u32(columns)
                            * vector_size_to_u32(rows);
                        (
                            size,
                            StructMemberType::Matrix {
                                columns: *columns,
                                rows: *rows,
                            },
                        )
                    }
                    _ => panic!(),
                };

                StructMemberLayout {
                    name: member.name.clone().unwrap(),
                    offset: member.offset,
                    size,
                    struct_type,
                }
            })
            .collect(),
        size: *size,
    }
}

// Convert a VectorSize enum to it's corresponding u32 value
fn vector_size_to_u32(size: &naga::VectorSize) -> u32 {
    match size {
        naga::VectorSize::Bi => 2,
        naga::VectorSize::Tri => 3,
        naga::VectorSize::Quad => 4,
    }
}

// Convert a naga shader stage to WGPU shader stage bitfield
fn naga_stage_to_wgpu_stage(
    stage: &naga::ShaderStage,
) -> wgpu::ShaderStages {
    match *stage {
        naga::ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
        naga::ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
        naga::ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
    }
}

// Fetch the BindingType of a naga Sampler
fn reflect_sampler(comparison: &bool) -> BindingType {
    BindingType::Sampler {
        sampler_binding: if *comparison {
            wgpu::SamplerBindingType::Comparison
        } else {
            wgpu::SamplerBindingType::Filtering
        },
    }
}

// Fetch the Bindingtype of a naga texture
fn reflect_texture(
    class: &naga::ImageClass,
    dim: &naga::ImageDimension,
) -> BindingType {
    BindingType::Texture {
        sample_type: match class {
            naga::ImageClass::Sampled { kind, multi: false } => {
                match kind {
                    naga::ScalarKind::Sint => {
                        wgpu::TextureSampleType::Sint
                    }
                    naga::ScalarKind::Uint => {
                        wgpu::TextureSampleType::Uint
                    }
                    naga::ScalarKind::Float => {
                        wgpu::TextureSampleType::Float {
                            filterable: true,
                        }
                    }
                    _ => panic!(),
                }
            }
            naga::ImageClass::Depth { multi: false } => {
                wgpu::TextureSampleType::Depth
            }

            _ => panic!(),
        },

        // Convert Naga image dimensions to WGPU texture dimensions
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
    }
}
