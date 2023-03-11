use std::{hash::Hash, sync::Arc};

use crate::{
    Compiled, FragmentModule, Graphics, ModuleKind, ShaderModule,
    VertexModule, TexelChannels,
};
use ahash::{AHashMap, AHashSet};
use naga::{AddressSpace, ResourceBinding, TypeInner};
use wgpu::TextureFormatFeatureFlags;

/*
Reflection only needs to do a few things.
It could either:
    1) Fetch binding index / set index for a specific resource (NEEDED)
    2) Check if a resource is shared (only in the case of vert - frag modules) (NEEDED)
    3) Validate shader UBO memory layout with the user given one
    4) Validate shader texture/sampler layout/format with the user given one
*/

/*
I have no idea if I should keep naga as a crate because it's kinda useless
The only thing it helps with is just making sure the source code can run on the backend and for
getting the set and binding indices so we don't have to specify them when defining resources in the compiler
 */


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
    pub visiblity: wgpu::ShaderStages,
}

// Push constant uniform data that we will fill field by field
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantLayout {
    pub name: String,
    pub visiblity: wgpu::ShaderStages,
    pub size: usize,
    pub alignment: usize,
}

// The type of BindingEntry. This is fetched from the given
// For now, only buffers, samplers, and texture are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindResourceType {
    // Either a Uniform buffer or a Storage buffer
    Buffer {
        size: usize,
        alignment: usize,
        storage: bool,
        read: bool,
        write: bool
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
}

// Reflect a vertex and fragment modules and create their respective pipeline layout
pub(super) fn merge_and_make_layout(
    vertex: &Compiled<VertexModule>,
    fragment: &Compiled<FragmentModule>,
    graphics: &Graphics,
) -> (Arc<ReflectedShader>, Arc<wgpu::PipelineLayout>) {
    // Convert the reflected module to a reflected shader
    let modules = &[vertex.reflected(), fragment.reflected()];
    let shader = merge_reflected_modules_to_shader(modules);

    // Convert the reflected shader to a layout
    let layout = create_pipeline_layout_from_shader(
        graphics,
        &shader,
        &[vertex.name(), fragment.name()],
    );
    (Arc::new(shader), layout)
}

// Merge multiple reflected modules to create a reflected shader
// This is private since the ordering of 'modules' is implementation defined
fn merge_reflected_modules_to_shader(
    modules: &[&ReflectedModule],
) -> ReflectedShader {
    // Stores multiple entries per set (max number of sets = 4)
    let mut groups: [Option<AHashMap<u32, BindResourceLayout>>; 4] =
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
                        if old.resource_type != merged.resource_type {
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
        value: &BindResourceLayout,
    ) -> wgpu::BindingType {
        match value.resource_type {
            BindResourceType::Buffer { size, alignment, storage, read, write } => {
                wgpu::BindingType::Buffer {
                    ty: match storage {
                        true => wgpu::BufferBindingType::Storage {
                            read_only: read && !write
                        },
                        false => wgpu::BufferBindingType::Uniform,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            }
            BindResourceType::Sampler { sampler_binding, .. } => {
                wgpu::BindingType::Sampler(sampler_binding)
            }
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
            stages: layout.visiblity,
            range: 0..(layout.size as u32),
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
pub(super) fn reflect_module<M: ShaderModule>(
    graphics: &Graphics,
    naga: &naga::Module,
    texture_formats: &super::TextureFormats,
    texture_dimensions: &super::TextureDimensions,
    uniform_buffer_pod_types: &super::UniformBufferPodTypes,
) -> ReflectedModule {
    let definitions = InternalDefinitions {
        texture_formats,
        texture_dimensions,
        uniform_buffer_pod_types,
    };

    let bind_group_layouts = reflect_binding_group::<M>(graphics, naga, &definitions);
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
fn reflect_binding_group<M: ShaderModule>(
    graphics: &Graphics,
    naga: &naga::Module,
    definitions: &InternalDefinitions,
) -> [Option<BindGroupLayout>; 4] {
    let mut bind_group_layouts: [Option<BindGroupLayout>; 4] =
        [None, None, None, None];
    let entries = reflect_binding_entries::<M>(graphics, naga, definitions);

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
fn reflect_binding_entries<M: ShaderModule>(
    graphics: &Graphics,
    naga: &naga::Module,
    definitions: &InternalDefinitions,
) -> Vec<BindResourceLayout> {
    let types = &naga.types;
    let vars = &naga.global_variables;

    // Iterate over the global variables and get their binding entry
    vars.iter()
        .filter_map(|(_, value)| {
            value.binding.as_ref().map(|_| value)
        })
        .filter(|value| {
            matches!(value.space, AddressSpace::Uniform) |
            matches!(value.space, AddressSpace::Storage { .. }) |
            matches!(value.space, AddressSpace::Handle)
        })
        .filter(|value| {
            types.get_handle(value.ty).is_ok()
        })
        .filter_map(|value| {
            // Get the type and address space of the variable
            let ResourceBinding { group, binding } =
                *value.binding.as_ref().unwrap();
            let typed = types.get_handle(value.ty).unwrap();
            let space = value.space;
            let type_inner = &typed.inner;

            let binding_type = match type_inner {
                // Uniform Buffers
                TypeInner::Struct {
                    members,
                    span: size,
                } => {
                    Some(reflect_buffer(members, types, size, value.space, definitions))
                }

                // Uniform Textures
                TypeInner::Image {
                    dim,
                    class,
                    arrayed,
                } => Some(reflect_texture(&value.name.as_ref().unwrap(), class, dim, graphics, definitions, *arrayed)),

                // Uniform Sampler
                TypeInner::Sampler { comparison } => {
                    Some(reflect_sampler(&value.name.as_ref().unwrap(), definitions, *comparison))
                }

                // This will get ignored later on
                // TODO: Is this okay?
                _ => None,
            };

            binding_type.map(|binding_type| BindResourceLayout {
                name: value.name.clone().unwrap(),
                binding,
                group,
                resource_type: binding_type,
                visiblity: kind_to_wgpu_stage(&M::kind()),
            })
        })
        .collect::<Vec<_>>()
}

// Fetches the used push constant of the given global variable
fn reflect_push_constant<M: ShaderModule>(
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

                output = Some(PushConstantLayout {
                    name,
                    visiblity: kind_to_wgpu_stage(&M::kind()),
                    alignment: /*alignment_of_inner_type(types, type_inner)*/0,
                    size: size_of_inner_type(type_inner),
                })
            }
            _ => {}
        }
    }

    output
}

// Get the size (in bytes) of the inner type
fn size_of_inner_type(type_inner: &TypeInner) -> usize {
    match type_inner {
        TypeInner::Struct { members, span } => {
            *span as usize
        },
        TypeInner::Scalar { kind, width } => todo!(),
        TypeInner::Vector { size, kind, width } => todo!(),
        TypeInner::Matrix { columns, rows, width } => todo!(),
        TypeInner::Array { base, size, stride } => todo!(),
        _ => panic!("Tried getting the size of a non-data type"),
    }
}

// Get the alignment (in bytes) of the inner type
fn alignment_of_inner_type(
    types: &naga::UniqueArena<naga::Type>,
    type_inner: &TypeInner
) -> usize {
    match type_inner {
        TypeInner::Struct { members, span } => {
            members.iter().map(|x| {
                let _type = types.get_handle(x.ty).unwrap();
                let inner = &_type.inner; 
                alignment_of_inner_type(types, inner)
            }).max().unwrap()
        },
        TypeInner::Scalar { kind, width } => todo!(),
        TypeInner::Vector { size, kind, width } => todo!(),
        TypeInner::Matrix { columns, rows, width } => todo!(),
        TypeInner::Array { base, size, stride } => todo!(),
        _ => panic!("Tried getting the alignment of a non-data type"),
    }
}

// Fetch the BindingType of a naga Struct (assuming it to be a buffer)
fn reflect_buffer(
    members: &Vec<naga::StructMember>,
    types: &naga::UniqueArena<naga::Type>,
    size: &u32,
    space: AddressSpace,
    definitions: &InternalDefinitions,
) -> BindResourceType {
    // TODO: Implement storage buffers

    BindResourceType::Buffer {
        size: *size as usize,
        alignment: 0,
        storage: false,
        read: true,
        write: false
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
fn reflect_sampler(
    name: &str,
    definitions: &InternalDefinitions,
    comparison: bool,
) -> BindResourceType {
    BindResourceType::Sampler {
        sampler_binding: wgpu::SamplerBindingType::Filtering,
        format: definitions.texture_formats.get(name).unwrap().format(),
    }
}

// Fetch the Bindingtype of a naga texture
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
            naga::ImageClass::Sampled { kind, multi } => {
                match kind {
                    naga::ScalarKind::Sint => {
                        wgpu::TextureSampleType::Sint
                    }
                    naga::ScalarKind::Uint => {
                        wgpu::TextureSampleType::Uint
                    }
                    naga::ScalarKind::Float => {
                        let adapter = graphics.adapter();
                        let info = definitions.texture_formats.get(name).unwrap();
                        let format = info.format();
                        let flags = adapter.get_texture_format_features(format).flags;
                        
                        let depth =  matches!(info.channels(), TexelChannels::Depth);    
            
                        if flags.contains(TextureFormatFeatureFlags::FILTERABLE) && !depth {
                            wgpu::TextureSampleType::Float { filterable: true }
                        } else {
                            wgpu::TextureSampleType::Float { filterable: false }
                        }
                    }
                    _ => panic!(),
                }
            }
            naga::ImageClass::Depth { multi } => todo!(),
            naga::ImageClass::Storage { format, access } => todo!(),
        },

        view_dimension: match dim {
            naga::ImageDimension::D1 => wgpu::TextureViewDimension::D1,
            naga::ImageDimension::D2 => wgpu::TextureViewDimension::D2,
            naga::ImageDimension::D3 => wgpu::TextureViewDimension::D3,
            naga::ImageDimension::Cube => wgpu::TextureViewDimension::Cube,
        },
        format: {
            let format = definitions.texture_formats.get(name).unwrap();
            format.format()
        },
        sampler_binding: {
            let adapter = graphics.adapter();
            let info = definitions.texture_formats.get(name).unwrap();
            let format = info.format();
            let flags = adapter.get_texture_format_features(format).flags;
            
            let depth =  matches!(info.channels(), TexelChannels::Depth);    

            if flags.contains(TextureFormatFeatureFlags::FILTERABLE) && !depth {
                wgpu::SamplerBindingType::Filtering
            } else {
                wgpu::SamplerBindingType::NonFiltering
            }
        },
    }
}