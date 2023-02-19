use crate::{Graphics, ShaderModule};
use ahash::{AHashMap, AHashSet};
use itertools::Itertools;
use naga::{AddressSpace, ResourceBinding, TypeInner};

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ReflectedShader {
    pub groups: Vec<BindGroupLayout>,
}

// This container stores all data related to reflected modules
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ReflectedModule {
    pub groups: Vec<BindGroupLayout>,
}

// A bind group contains one or more bind entries
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BindGroupLayout {
    pub entries: Vec<BindEntryLayout>,
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
    // TODO: Handle gaps in the group bindings (set = 0, hops over to set = 2, gap at set = 1)
    let mut entries: AHashMap<u32, AHashMap<u32, BindEntryLayout>> = AHashMap::new();
    
    for module in modules {
        for (index, group) in module.groups.iter().enumerate() {
            let set = entries.entry(index as u32).or_default();
            
            for entry in group.entries.iter() {
                let layout= set.entry(entry.binding).or_insert(entry.clone());
                
                if entry.binding_type != layout.binding_type {
                    panic!();
                }

                layout.visiblity.insert(entry.visiblity);
            }
        }
    }

    // TODO: Handle gaps in the group bindings (set = 0, hops over to set = 2, gap at set = 1)
    let groups = entries.iter().map(|(_, set)| {
        BindGroupLayout {
            entries: set.iter().map(|(_, x)| x).cloned().collect(),
        }
    }).collect::<Vec<_>>();

    dbg!(&groups);
    
    ReflectedShader { groups }
}

// Convert a given reflected shader to a pipeline layout (by creating it)
pub fn create_pipeline_layout_from_shader(
    graphics: &Graphics,
    shader: &ReflectedShader,
) -> wgpu::PipelineLayout {
    // TODO: Handle gaps in the group bindings (set = 0, hops over to set = 2, gap at set = 1)

    // Merge each binding entry by group (itertools)
    let bind_group_layout_entries: Vec<
        Vec<wgpu::BindGroupLayoutEntry>,
    > = shader
        .groups
        .iter()
        .map(|group| {
            // Convert each entry from this group to a WGPU BindGroupLayoutEntry
            group
                .entries
                .iter()
                .map(|value| wgpu::BindGroupLayoutEntry {
                    binding: value.binding,
                    visibility: value.visiblity,
                    ty: match value.binding_type {
                        BindingType::Buffer {
                            buffer_binding,
                            ..
                        } => wgpu::BindingType::Buffer {
                            ty: buffer_binding,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        BindingType::Sampler { sampler_binding } => {
                            wgpu::BindingType::Sampler(
                                sampler_binding,
                            )
                        }
                        BindingType::Texture {
                            sample_type,
                            view_dimension,
                        } => wgpu::BindingType::Texture {
                            sample_type,
                            view_dimension,
                            multisampled: false,
                        },
                    },
                    count: None,
                })
                .collect()
        })
        .collect();

    // Create the BindGroupLayoutDescriptor for the BindgGroupEntries
    let bind_group_layout_descriptors = bind_group_layout_entries
        .iter()
        .map(|entries| wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        })
        .collect::<Vec<_>>();
    dbg!(&bind_group_layout_descriptors);

    // TODO: Validate the bindings and groups

    // Create the bind group layouts from the corresponding descriptors
    // TODO: Cache reusable bind group layouts
    let bind_group_layouts = bind_group_layout_descriptors
        .iter()
        .map(|desc| graphics.device().create_bind_group_layout(desc))
        .collect::<Vec<_>>();
    let bind_group_layouts =
        bind_group_layouts.iter().collect::<Vec<_>>();

    // Create the pipeline layout
    graphics.device().create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        },
    )
}

// Reflect a naga module's bindings and constants
pub fn reflect_module<M: ShaderModule>(
    naga: &naga::Module,
) -> ReflectedModule {
    let groups = reflect_binding_group::<M>(naga);
    ReflectedModule {
        groups,
    }
}

// Fetches the used binding groups of a given naga module
pub fn reflect_binding_group<M: ShaderModule>(
    naga: &naga::Module,
) -> Vec<BindGroupLayout> {
    let entries = reflect_binding_entries::<M>(naga);
    let grouped = entries.into_iter().group_by(|x| x.group);
    grouped
        .into_iter()
        .map(|(index, entries)| BindGroupLayout {
            entries: entries.collect(),
        })
        .collect()
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
