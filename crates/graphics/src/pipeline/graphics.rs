use std::{marker::PhantomData, sync::Arc, num::NonZeroU64};

use ahash::AHashMap;
use itertools::Itertools;
use wgpu::{PrimitiveState, VertexStepMode};

use crate::{Shader, Graphics, PipelineInitializationError, DepthConfig, StencilConfig, BlendConfig, PrimitiveConfig, VertexConfig, DepthStencilLayout, ColorLayout, VertexInfo, VertexInputInfo, PipelineBindingsError};

// Wrapper around a WGPU render pipeline just to help me instantiate them
pub struct GraphicsPipeline<C: ColorLayout, DS: DepthStencilLayout> {
    pipeline: wgpu::RenderPipeline,

    // Immutable data set at build time
    depth_config: Option<DepthConfig>,
    stencil_config: Option<StencilConfig>,
    //blend_config: Option<BlendConfig>,
    vertex_config: VertexConfig,
    primitive_config: PrimitiveConfig,
    _phantom: PhantomData<C>,
    _phantom2: PhantomData<DS>,

    // Keep the shader modules alive
    shader: Shader,

    // Keep the graphics API alive
    graphics: Graphics,
}

// Initialization of the graphics pipeline
impl<C: ColorLayout, DS: DepthStencilLayout> GraphicsPipeline<C, DS> {
    // Create a new pipeline with the specified configs
    pub fn new(
        graphics: &Graphics,
        depth_config: Option<DepthConfig>,
        stencil_config: Option<StencilConfig>,
        vertex_config: VertexConfig,
        primitive_config: PrimitiveConfig,
        shader: &Shader,
    ) -> Result<Self, PipelineInitializationError> {
        // If stencil/depth is enabled, make sure the layout matches up
        let stencil_config_enabled = DS::is_stencil_enabled();
        let depth_config_enabled = DS::is_depth_enabled();

        // Check if the DepthStencilLayout contains a stencil format, return errors if appropriate
        if stencil_config_enabled && stencil_config.is_none() {
            return Err(PipelineInitializationError::MissingStencilConfig);
        }

        // Check if the DepthStencilLayout contains a depth format, return errors if appropriate
        if depth_config_enabled && depth_config.is_none() {
            return Err(PipelineInitializationError::MissingStencilConfig);
        }

        // Get all the configuration settings required for the RenderPipeline 
        let depth_stencil = depth_stencil_config_to_state::<DS>(&depth_config, &stencil_config);
        let attributes = vertex_config_to_vertex_attributes(&vertex_config);
        let attributes = attributes.iter().map(|x| x.as_slice()).collect();
        let buffers = vertex_config_to_buffer_layout(&vertex_config, attributes);
        let targets = color_layout_to_color_target_state::<C>();
        let primitive = primitive_config_to_state(primitive_config);
        let layout = shader_to_pipeline_layout(&graphics, &shader);
        let layout = Some(layout.map_err(PipelineInitializationError::InvalidBindings)?);

        // Create the WGPU pipeline using the given configuration
        let pipeline = graphics.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: layout.as_ref(),
            vertex: wgpu::VertexState {
                module: shader.vertex().module(),
                entry_point: shader.vertex().entry_point().unwrap(),
                buffers: &buffers,
            },
            primitive,
            depth_stencil,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader.fragment().module(),
                entry_point: shader.vertex().entry_point().unwrap(),
                targets: &targets,
            }),
            multiview: None,
        });

        Ok(Self {
            pipeline,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            shader: shader.clone(),
            graphics: graphics.clone(),
            depth_config,
            stencil_config,
            vertex_config,
            primitive_config,
        })
    }
}

// Fetch the reflected shader data (from shaders modules) and merge it
fn shader_to_pipeline_layout(graphics: &Graphics, shader: &Shader) -> Result<wgpu::PipelineLayout, PipelineBindingsError> {
    use naga::{TypeInner, ResourceBinding};

    #[derive(Debug)]
    pub struct BindingEntry {
        pub name: String,
        pub binding: u32,
        pub group: u32,
        pub binding_type: BindingType,
        pub visiblity: wgpu::ShaderStages,
    }

    #[derive(Debug)]
    pub struct StructMember {
        pub name: String,
        pub offset: u32,
        pub size: u32,
        pub struct_type: StructMemberType,
    }

    #[derive(Debug)]
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

    #[derive(Debug)]
    pub enum BindingType {
        Buffer {
            buffer_binding: wgpu::BufferBindingType,
            members: Vec<StructMember>,
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

    let naga = shader.vertex().naga();
    dbg!(naga);
    let types = &naga.types;
    let vars = &naga.global_variables;
    
    // Iterate over the global variables and get their binding entry
    let binding_entries = vars.iter().filter_map(|(_, value)| {
        value.binding.as_ref().map(|_| value)
    }).map(|value| {
        let ResourceBinding {
            group,
            binding,
        } = *value.binding.as_ref().unwrap();

        let typed = types.get_handle(value.ty).unwrap();
        let type_inner = &typed.inner;
        let space = value.space;

        let binding_type = match &type_inner {
            // Uniform Buffers
            TypeInner::Struct { members, span: size } if space == naga::AddressSpace::Uniform => {
                BindingType::Buffer {
                    buffer_binding: wgpu::BufferBindingType::Uniform,
                    members: members.iter().map(|member| {
                        let type_inner = &types.get_handle(member.ty).unwrap().inner;
                        let (size, struct_type) = match type_inner {
                            TypeInner::Scalar { kind, width } => {
                                (*width as u32, StructMemberType::Scalar { kind: *kind })
                            },
                            TypeInner::Vector { size, kind, width } => {
                                (*width as u32 * match size {
                                    naga::VectorSize::Bi => 2,
                                    naga::VectorSize::Tri => 3,
                                    naga::VectorSize::Quad => 4,
                                }, StructMemberType::Vector { size: *size, kind: *kind })
                            },
                            TypeInner::Matrix { columns, rows, width } => {
                                (*width as u32 * match columns {
                                    naga::VectorSize::Bi => 2,
                                    naga::VectorSize::Tri => 3,
                                    naga::VectorSize::Quad => 4,
                                } * match rows {
                                    naga::VectorSize::Bi => 2,
                                    naga::VectorSize::Tri => 3,
                                    naga::VectorSize::Quad => 4,
                                }, StructMemberType::Matrix { columns: *columns, rows: *rows })
                            },
                            _ => panic!()
                        };
                        
                        StructMember {
                            name: member.name.clone().unwrap(),
                            offset: member.offset,
                            size,
                            struct_type,
                        }
                    }).collect(),
                    size: *size,
                }
            },

            // Uniform Textures
            TypeInner::Image { dim, class, .. } if space == naga::AddressSpace::Handle => {
                BindingType::Texture {
                    sample_type: match class {
                        naga::ImageClass::Sampled { kind, multi: false } => {
                            match kind {
                                naga::ScalarKind::Sint => wgpu::TextureSampleType::Sint,
                                naga::ScalarKind::Uint => wgpu::TextureSampleType::Uint,
                                naga::ScalarKind::Float => wgpu::TextureSampleType::Float { filterable: true },
                                _ => panic!()
                            }
                        },
                        naga::ImageClass::Depth { multi: false } => {
                            wgpu::TextureSampleType::Depth
                        },

                        _ => panic!()
                    },

                    // Convert Naga image dimensions to WGPU texture dimensions
                    view_dimension: match dim {
                        naga::ImageDimension::D1 => wgpu::TextureViewDimension::D1,
                        naga::ImageDimension::D2 => wgpu::TextureViewDimension::D2,
                        naga::ImageDimension::D3 => wgpu::TextureViewDimension::D3,
                        naga::ImageDimension::Cube => wgpu::TextureViewDimension::Cube,
                    },
                }
            },

            // Uniform Sampler
            TypeInner::Sampler { comparison } if space == naga::AddressSpace::Handle => {
                BindingType::Sampler {
                    sampler_binding: if *comparison {
                        wgpu::SamplerBindingType::Comparison
                    } else { wgpu::SamplerBindingType::Filtering }
                }
            },
            _ => todo!()
        };

        BindingEntry {
            name: value.name.clone().unwrap(),
            binding,
            group,
            binding_type,
            visiblity: wgpu::ShaderStages::all(),
        }
    });

    
    // Merge each binding entry by group (itertools)
    let bind_group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>> = binding_entries.group_by(|value| {
        value.group
    }).into_iter().map(|(group, values)| {
        // Convert each entry from this group to a WGPU BindGroupLayoutEntry 
        values.map(|value| {
            wgpu::BindGroupLayoutEntry {
                binding: value.binding,
                visibility: value.visiblity,
                ty: match value.binding_type {
                    BindingType::Buffer { buffer_binding, .. } => wgpu::BindingType::Buffer {
                        ty: buffer_binding,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    BindingType::Sampler { sampler_binding } => wgpu::BindingType::Sampler(sampler_binding),
                    BindingType::Texture { sample_type, view_dimension } => wgpu::BindingType::Texture {
                        sample_type,
                        view_dimension,
                        multisampled: false
                    },
                },
                count: None,
            }
        }).collect()
    }).collect();

    // Create the BindGroupLayoutDescriptor for the BindgGroupEntries
    let bind_group_layout_descriptors = bind_group_layout_entries.iter().map(|entries| {
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        }
    }).collect::<Vec<_>>();
    dbg!(&bind_group_layout_descriptors);

    // TODO: Validate the bindings and groups
    
    // Create the bind group layouts from the corresponding descriptors
    let bind_group_layouts = bind_group_layout_descriptors.iter().map(|desc| {
        graphics.device().create_bind_group_layout(desc)
    }).collect::<Vec<_>>();
    let bind_group_layouts = bind_group_layouts
        .iter()
        .collect::<Vec<_>>();

    Ok(graphics.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &bind_group_layouts,
        push_constant_ranges: &[],
    }))
}

// Convert the given vertex config to the vertex attributes used by the vertex buffer layout
fn vertex_config_to_vertex_attributes(vertex_config: &VertexConfig) -> Vec<Vec<wgpu::VertexAttribute>> {
    // TODO: Implement custom shader location + shader location checking + mapping
    vertex_config.inputs.iter().enumerate().map(|(i, input)| {
        vec![wgpu::VertexAttribute {
            format: input.vertex_info().format(),
            offset: 0,
            shader_location: i as u32,
        }]
    }).collect()
}

// Conver the given vertex config to internally used buffer layout
fn vertex_config_to_buffer_layout<'a>(vertex_config: &VertexConfig, attributes: Vec<&'a [wgpu::VertexAttribute]>) -> Vec<wgpu::VertexBufferLayout<'a>> {
    vertex_config.inputs.iter().enumerate().map(|(index, input)| {
        let attribute = &attributes[index];
        
        wgpu::VertexBufferLayout {
            array_stride: input.vertex_info().size() as u64,
            step_mode: input.step_mode(),
            attributes: attribute,
        }
    }).collect()
}

// Conver the statically typed color layout to the color target states needed for the fragment field
fn color_layout_to_color_target_state<C: ColorLayout>() -> Vec<Option<wgpu::ColorTargetState>> {
    let targets = C::layout_info().into_iter().map(|info| Some(wgpu::ColorTargetState {
        format: info.format(),
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })).collect::<Vec<_>>();
    targets
}

// Convert the depth and stencil config to the DepthStencilState
fn depth_stencil_config_to_state<DS: DepthStencilLayout>(
    depth_config: &Option<DepthConfig>,
    stencil_config: &Option<StencilConfig>
) -> Option<wgpu::DepthStencilState> {
    // Get the depth bias state for the DepthStencilState
    let bias = if let Some(depth_config) = depth_config {
        wgpu::DepthBiasState {
            constant: depth_config.depth_bias_constant,
            slope_scale: depth_config.depth_bias_slope_scale,
            clamp: depth_config.depth_bias_clamp,
        }
    } else {
        wgpu::DepthBiasState::default()
    };

    // Get stencil, depth comparison function, depth write
    let stencil = stencil_config.as_ref().cloned().unwrap_or_default();
    let depth_compare = depth_config.map(|dc| dc.compare).unwrap_or(wgpu::CompareFunction::Never);
    let depth_write_enabled = depth_config.map(|dc| dc.write_enabled).unwrap_or_default();
    DS::info().map(|format| wgpu::DepthStencilState {
        format: format.format(),
        depth_write_enabled,
        depth_compare,
        stencil,
        bias,
    })
}

// Convert the primitive config to primitive state
fn primitive_config_to_state(primitive_config: PrimitiveConfig) -> PrimitiveState {
    let topology = match primitive_config {
        PrimitiveConfig::Triangles { .. } => wgpu::PrimitiveTopology::TriangleList,
        PrimitiveConfig::Lines { .. } => wgpu::PrimitiveTopology::LineList,
        PrimitiveConfig::Points => wgpu::PrimitiveTopology::PointList,
    };

    let front_face = match primitive_config {
        PrimitiveConfig::Triangles { winding_order, .. } => winding_order,
        _ => wgpu::FrontFace::Cw,
    };

    let cull_mode = match primitive_config {
        PrimitiveConfig::Triangles { cull_face, .. } => cull_face,
        _ => None
    };

    let polygon_mode = match primitive_config {
        PrimitiveConfig::Triangles { wireframe: true, .. } => wgpu::PolygonMode::Line,
        _ => wgpu::PolygonMode::Fill
    };

    wgpu::PrimitiveState {
        topology,
        strip_index_format: None,
        front_face,
        cull_mode,
        unclipped_depth: false,
        polygon_mode,
        conservative: false,
    }
}

impl<C: ColorLayout, DS: DepthStencilLayout> GraphicsPipeline<C, DS> {
    // Get the underlying raw WGPU pipeline
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    // Get the depth config used when creating this pipeline
    pub fn depth_config(&self) -> Option<&DepthConfig> {
        self.depth_config.as_ref()
    }

    // Get the stencil config used when creating this pipeline
    pub fn stencil_config(&self) -> Option<&StencilConfig> {
        self.stencil_config.as_ref()
    }

    /*
    // Get the blend config used when creating this pipeline
    pub fn blend_config(&self) -> Option<&BlendConfig> {
        self.blend_config.as_ref()
    }
    */

    // Get the vertex config used when creating this pipeline
    pub fn vertex_config(&self) -> &VertexConfig {
        &self.vertex_config
    }

    // Get the internally used shader for this graphics pipeline
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    // Get the primitive config used when creating this pipeline
    pub fn primitive_config(&self) -> &PrimitiveConfig {
        &self.primitive_config
    }
}