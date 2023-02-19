use crate::ShaderModule;
use ahash::AHashMap;
use itertools::Itertools;
use naga::{AddressSpace, ResourceBinding, TypeInner};

// Stores data related to reflected shader (vertex + frag)
// This also creates a valid pipeline layout to be used in graphics pipelines
pub struct ReflectedShader {
    pub groups: Vec<BindGroupLayout>,
    pub modules: wgpu::ShaderStages,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

// This container stores all data related to reflected modules
#[derive(Debug, Clone, PartialEq)]
pub struct ReflectedModule {
    pub groups: Vec<BindGroupLayout>,
    pub module: naga::ShaderStage,
}

// A bind group contains one or more bind entries
#[derive(Debug, Clone, PartialEq)]
pub struct BindGroupLayout {
    pub index: u32,
    pub entries: Vec<BindEntryLayout>,
}

// A binding entry is a single binding resource from within a group
// Eg. a uniform buffer, a sampler, a texture, or storage buffer
#[derive(Debug, Clone, PartialEq)]
pub struct BindEntryLayout {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub binding_type: BindingType,
    pub visiblity: wgpu::ShaderStages,
}

// The type of BindingEntry.
// For now, only buffers, samplers, and texture are supported
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct StructMemberLayout {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub struct_type: StructMemberType,
}

// Types of buffer structure fields
#[derive(Debug, Clone, Copy, PartialEq)]
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
pub fn merge_reflected_module(
    modules: &[&ReflectedModule],
) -> ReflectedShader {
    todo!()
}

// Reflect a naga module's bindings and constants
pub fn reflect_module<M: ShaderModule>(
    naga: &naga::Module,
) -> ReflectedModule {
    let groups = reflect_binding_group::<M>(naga);
    ReflectedModule {
        groups,
        module: M::stage(),
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
            index,
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
