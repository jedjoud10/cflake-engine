use std::marker::PhantomData;

use wgpu::PrimitiveState;

use crate::{Shader, Graphics, PipelineInitializationError, DepthConfig, StencilConfig, BlendConfig, PrimitiveConfig, BindingConfig, VertexConfig, DepthStencilLayout, ColorLayout};

// Wrapper around a WGPU render pipeline just to help me instantiate them
pub struct GraphicsPipeline<C: ColorLayout, DS: DepthStencilLayout> {
    pipeline: wgpu::RenderPipeline,

    // Immutable data set at build time
    depth_config: DepthConfig,
    stencil_config: StencilConfig,
    blend_config: BlendConfig,
    vertex_config: VertexConfig,
    primitive_config: PrimitiveConfig,
    binding_config: BindingConfig,
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
        blend_config: Option<BlendConfig>,
        vertex_config: VertexConfig,
        primitive_config: PrimitiveConfig,
        binding_config: Option<BindingConfig>,
        shader: Shader,
    ) -> Result<Self, PipelineInitializationError> {
        // If stencil/depth is enabled, make sure the layout matches up
        let stencil_config_enabled = stencil_config.is_some();
        let depth_config_enabled = depth_config.is_some();

        // Check if the DepthStencilLayout contains a stencil format
        if stencil_config_enabled {
            
        }

        // Check if the DepthStencilLayout contains a depth format
        if depth_config_enabled {

        }

        // Create a depth stencil state if either the depth config or stencil config are enabled
        let depth_stencil_state = (depth_config_enabled || stencil_config_enabled).then(|| {
            wgpu::DepthStencilState {
                format: todo!(),
                depth_write_enabled: todo!(),
                depth_compare: todo!(),
                stencil: todo!(),
                bias: todo!(),
            }
        });

        // Create a depth stencil state for the depth config
        
        
        // Create a stencil state for the stencil config
        
        // Create the WGPU primitive state
        let primitive = primitive_config_to_state(primitive_config);

        // Create the WGPU pipeline using the given configuration
        let pipeline = graphics.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: todo!(),
            primitive,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: todo!(),
                depth_write_enabled: todo!(),
                depth_compare: todo!(),
                stencil: wgpu::StencilState {
                    front: todo!(),
                    back: todo!(),
                    read_mask: todo!(),
                    write_mask: todo!(),
                },
                bias: wgpu::DepthBiasState {
                    constant: todo!(),
                    slope_scale: todo!(),
                    clamp: todo!(),
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: todo!(),
            multiview: None,
        });


    }
}

// Convert the primitive config to primitive state
fn primitive_config_to_state(primitive_config: PrimitiveConfig) -> PrimitiveState {
    let topology = match primitive_config {
        PrimitiveConfig::Triangles { .. } => wgpu::PrimitiveTopology::TriangleList,
        PrimitiveConfig::Lines { .. } => wgpu::PrimitiveTopology::LineList,
        PrimitiveConfig::Points => wgpu::PrimitiveTopology::PointList,
    };

    let front_face = match primitive_config {
        PrimitiveConfig::Triangles { winding_order, cull_face, wireframe } => winding_order,
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
        strip_index_format: Some(wgpu::IndexFormat::Uint32),
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
    pub fn depth_config(&self) -> &DepthConfig {
        &self.depth_config
    }

    // Get the stencil config used when creating this pipeline
    pub fn stencil_config(&self) -> &StencilConfig {
        &self.stencil_config
    }

    // Get the blend config used when creating this pipeline
    pub fn blend_config(&self) -> &BlendConfig {
        &self.blend_config
    }

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

    // Get the binding config used when creating this pipeline
    pub fn binding_config(&self) -> &BindingConfig {
        &self.binding_config
    }
}