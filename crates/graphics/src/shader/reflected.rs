use std::{hash::Hash, sync::Arc};

use crate::{
    Compiled, FragmentModule, Graphics, ModuleKind, ShaderModule,
    VertexModule,
};
use ahash::{AHashMap, AHashSet};
use naga::{AddressSpace, ResourceBinding, TypeInner};

// This container stores all data related to reflected shaders
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedShader {
    pub last_valid_bind_group_layout: usize,
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
    pub push_constant_layouts: [Option<PushConstantLayout>; 2],
}

// This container stores all data related to reflected modules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReflectedModule {
    pub last_valid_bind_group_layout: usize,
    pub bind_group_layouts: [Option<BindGroupLayout>; 4],
    pub push_constant: Option<PushConstantLayout>,
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

// Push constant uniform data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantLayout {
    pub name: String,
    pub stages: wgpu::ShaderStages,
    pub members: Vec<StructMemberLayout>,
    pub size: u32,
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

// Reflect a vertex and fragment modules and create their respective pipeline layout
pub fn merge_and_make_layout(
    vertex: &Compiled<VertexModule>,
    fragment: &Compiled<FragmentModule>,
    graphics: &Graphics,
) -> (ReflectedShader, Arc<wgpu::PipelineLayout>) {
    // Convert the reflected module to a reflected shader
    let modules = &[vertex.reflected(), fragment.reflected()];
    let shader = merge_reflected_modules_to_shader(modules);

    // Convert the reflected shader to a layout
    let layout = create_pipeline_layout_from_shader(
        graphics,
        &shader,
        &[vertex.file_name(), fragment.file_name()],
    );
    (shader, layout)
}

// Merge multiple reflected modules to create a reflected shader
// This is private since the ordering of 'modules' is implementation defined
fn merge_reflected_modules_to_shader(
    modules: &[&ReflectedModule],
) -> ReflectedShader {
    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: [Option<AHashMap<u32, BindEntryLayout>>; 4] =
        [None, None, None, None];

    // Stores mutliple push constants for each module (at max we will have 2 modules)
    let mut push_constant_layouts: [Option<PushConstantLayout>; 2] =
        [None, None];

    // Keep track of the last valid bind group layout index (max)
    let mut last_valid_bind_group_layout = 0;

    // Merge different modules into a ReflectedShader
    for (module_index, module) in modules.iter().enumerate() {
        // Add the bind group push constant layout (if it exists)
        push_constant_layouts[module_index] =
            module.push_constant.clone();
        last_valid_bind_group_layout = last_valid_bind_group_layout
            .max(module.last_valid_bind_group_layout);

        // Merrge bind groups and their entries
        for (group_index, bind_group_layout) in
            module.bind_group_layouts.iter().enumerate()
        {
            // Skip this bind group if it was hopped over in the shader
            let Some(bind_group_layout) = bind_group_layout else {
                continue;
            };

            // This bind group MUST contains at least ONE entry
            if bind_group_layout.bind_entry_layouts.len() == 0 {
                panic!("Bind group MUST contain at least ONE entry");
            }

            // Get the merged group layout and merged group entry layouts
            let merged_group_layout =
                &mut groups[group_index as usize];
            let merged_group_entry_layouts = merged_group_layout
                .get_or_insert_with(|| Default::default());

            // Merge each entry for this group individually
            for bind_entry_layout in
                bind_group_layout.bind_entry_layouts.iter()
            {
                match merged_group_entry_layouts
                    .entry(bind_entry_layout.binding)
                {
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
                        if old.binding_type != merged.binding_type {
                            panic!("Not the same layout");
                        }

                        // Merge the visibility to allow more modules to access this entry
                        merged.visiblity.insert(old.visiblity);
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

    // Convert the entries back into bind groups
    let groups: [Option<BindGroupLayout>; 4] = groups
        .into_iter()
        .map(|entries| {
            entries.map(|entries| BindGroupLayout {
                bind_entry_layouts: entries
                    .into_iter()
                    .map(|(_, x)| x)
                    .collect(),
            })
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    ReflectedShader {
        bind_group_layouts: groups,
        push_constant_layouts,
        last_valid_bind_group_layout,
    }
}

// Convert a given reflected shader to a pipeline layout (by creating it)
// This is private since the ordering of 'names' is implementation defined
fn create_pipeline_layout_from_shader(
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

    // Convert the push constant range layouts to push constant ranges
    let push_constant_ranges = shader
        .push_constant_layouts
        .iter()
        .filter_map(|x| x.as_ref())
        .map(|layout| wgpu::PushConstantRange {
            stages: layout.stages,
            range: 0..layout.size,
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
    layout
}

// Reflect a naga module's bindings and constants
pub fn reflect_module<M: ShaderModule>(
    naga: &naga::Module,
) -> ReflectedModule {
    let bind_group_layouts = reflect_binding_group::<M>(naga);
    let push_constant = reflect_push_constant::<M>(naga);

    let last_valid_bind_group_layout = bind_group_layouts
        .iter()
        .rposition(|x| x.is_some())
        .unwrap_or_default();

    ReflectedModule {
        bind_group_layouts,
        last_valid_bind_group_layout,
        push_constant,
    }
}

// Fetches the used binding groups of a given naga module
pub fn reflect_binding_group<M: ShaderModule>(
    naga: &naga::Module,
) -> [Option<BindGroupLayout>; 4] {
    let mut bind_group_layouts: [Option<BindGroupLayout>; 4] =
        [None, None, None, None];
    let entries = reflect_binding_entries::<M>(naga);

    // Merge the binding entries into their respective bind group layouts
    for bind_entry_layout in entries {
        let bind_group_layout =
            &mut bind_group_layouts[bind_entry_layout.group as usize];
        let bind_group_layout =
            bind_group_layout.get_or_insert_with(|| {
                BindGroupLayout {
                    bind_entry_layouts: Vec::new(),
                }
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
        .filter(|value| {
            value.space == AddressSpace::Uniform
                || value.space == AddressSpace::Handle
        })
        .map(|value| {
            // Get the type and address space of the variable
            let ResourceBinding { group, binding } =
                *value.binding.as_ref().unwrap();
            let typed = types.get_handle(value.ty).unwrap();
            let type_inner = &typed.inner;

            //reflect_bind_entry::<M>(value, types);
            let binding_type = match type_inner {
                // Uniform Buffers
                TypeInner::Struct {
                    members,
                    span: size,
                } => {
                    reflect_buffer(members, types, size, value.space)
                }

                // Uniform Textures
                TypeInner::Image { dim, class, arrayed  } => {
                    reflect_texture(class, dim)
                }

                // Uniform Sampler
                TypeInner::Sampler { comparison } => {
                    reflect_sampler(comparison)
                }
                _ => panic!("Not supported"),
            };

            BindEntryLayout {
                name: value.name.clone().unwrap(),
                binding,
                group,
                binding_type,
                visiblity: kind_to_wgpu_stage(&M::kind()),
            }
        })
        .collect::<Vec<_>>()
}

// Fetches the used push constant of the given global variable
pub fn reflect_push_constant<M: ShaderModule>(
    naga: &naga::Module,
) -> Option<PushConstantLayout> {
    // Get the type and address space of the variable
    let types = &naga.types;
    let vars = &naga.global_variables;

    // The push constant layout that we will return
    let mut output: Option<PushConstantLayout> = None;

    // Try to find a push constant that we use
    for (_, value) in vars.iter() {
        match value.space {
            AddressSpace::PushConstant => {
                let typed = types.get_handle(value.ty).unwrap();
                let type_inner = &typed.inner;
                let name = value.name.clone().unwrap().clone();

                let (members, size) = match type_inner {
                    TypeInner::Struct { members, span } => {
                        (members, *span)
                    }
                    _ => panic!(""),
                };

                let members =
                    reflect_struct_member_layouts(members, types);

                output = Some(PushConstantLayout {
                    name,
                    members,
                    size,
                    stages: kind_to_wgpu_stage(&M::kind()),
                })
            }
            _ => {}
        }
    }

    output
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
        members: reflect_struct_member_layouts(members, types),
        size: *size,
    }
}

// Fetch teh struct layout of the struct member layout
fn reflect_struct_member_layouts(
    members: &Vec<naga::StructMember>,
    types: &naga::UniqueArena<naga::Type>,
) -> Vec<StructMemberLayout> {
    members
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
        .collect()
}

// Convert a VectorSize enum to it's corresponding u32 value
fn vector_size_to_u32(size: &naga::VectorSize) -> u32 {
    match size {
        naga::VectorSize::Bi => 2,
        naga::VectorSize::Tri => 3,
        naga::VectorSize::Quad => 4,
    }
}

// Convert a module ind to Naga shader stage
pub(super) fn kind_to_naga_stage(
    kind: &ModuleKind,
) -> naga::ShaderStage {
    match *kind {
        ModuleKind::Vertex => naga::ShaderStage::Vertex,
        ModuleKind::Fragment => naga::ShaderStage::Fragment,
        ModuleKind::Compute => naga::ShaderStage::Compute,
    }
}

// Convert a module kind to WGPU shader stage bitfield
pub(super) fn kind_to_wgpu_stage(
    kind: &ModuleKind,
) -> wgpu::ShaderStages {
    match *kind {
        ModuleKind::Vertex => wgpu::ShaderStages::VERTEX,
        ModuleKind::Fragment => wgpu::ShaderStages::FRAGMENT,
        ModuleKind::Compute => wgpu::ShaderStages::COMPUTE,
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
                panic!()
            }
            naga::ImageClass::Storage { format, access } => {
                panic!()
            }

            _ => {
                panic!()
            }
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
