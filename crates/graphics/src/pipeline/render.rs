use std::{marker::PhantomData, num::NonZeroU64, sync::Arc};

use ahash::AHashMap;
use itertools::Itertools;
use wgpu::{BlendState, PrimitiveState, VertexStepMode};

use crate::{
    BlendConfig, ColorLayout, DepthConfig, DepthStencilLayout, Graphics,
    PipelineInitializationError, PrimitiveConfig, Shader, StencilConfig, VertexConfig, VertexInfo,
    VertexInputInfo,
};

// Wrapper around a WGPU render pipeline just to help me instantiate them
pub struct RenderPipeline<C: ColorLayout, DS: DepthStencilLayout> {
    pipeline: wgpu::RenderPipeline,

    // Immutable data set at build time
    depth_config: Option<DepthConfig>,
    stencil_config: Option<StencilConfig>,
    blend_config: Option<BlendConfig<C>>,
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
impl<C: ColorLayout, DS: DepthStencilLayout> RenderPipeline<C, DS> {
    // Create a new pipeline with the specified configs
    pub fn new(
        graphics: &Graphics,
        depth_config: Option<DepthConfig>,
        stencil_config: Option<StencilConfig>,
        blend_config: Option<BlendConfig<C>>,
        vertex_config: VertexConfig,
        primitive_config: PrimitiveConfig,
        shader: &Shader,
    ) -> Result<Self, PipelineInitializationError> {
        // If stencil/depth is enabled, make sure the layout matches up
        let stencil_config_enabled = DS::is_stencil_enabled();
        let depth_config_enabled = DS::is_depth_enabled();

        // Check if the DepthStencilLayout does not contain a stencil format, return warning if appropriate
        if !stencil_config_enabled && stencil_config.is_none() {
            log::warn!(
                "Tried using stencil config for a graphics pipeline without a stencil texel"
            );
        }

        // Check if the DepthStencilLayout does not contain a depth format, return warning if appropriate
        if !depth_config_enabled && depth_config.is_none() {
            log::warn!("Tried using depth config for a graphics pipeline without a depth texel");
        }

        // Get all the configuration settings required for the RenderPipeline
        let depth_stencil = depth_stencil_config_to_state::<DS>(&depth_config, &stencil_config);
        let attributes = vertex_config_to_vertex_attributes(&vertex_config);
        let attributes = attributes.iter().map(|x| x.as_slice()).collect();
        let buffers = vertex_config_to_buffer_layout(&vertex_config, attributes);
        let targets = color_layout_to_color_target_state::<C>(&blend_config);
        let primitive = primitive_config_to_state(primitive_config);
        let layout = &shader.layout;

        // Create a name for the render pipeline
        let name = format!(
            "render-pipeline-{:?}",
            [shader.vertex().name(), shader.fragment().name()]
        );

        // Create the WGPU pipeline using the given configuration
        let pipeline = graphics
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&name),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader.vertex().module(),
                    entry_point: "main",
                    buffers: &buffers,
                },
                primitive,
                depth_stencil,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: shader.fragment().module(),
                    entry_point: "main",
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
            blend_config,
        })
    }
}

// Convert the given vertex config to the vertex attributes used by the vertex buffer layout
fn vertex_config_to_vertex_attributes(
    vertex_config: &VertexConfig,
) -> Vec<Vec<wgpu::VertexAttribute>> {
    // TODO: Implement custom shader location + shader location checking + mapping +
    vertex_config
        .inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            vec![wgpu::VertexAttribute {
                format: input.vertex_info().format(),
                offset: 0,
                shader_location: i as u32,
            }]
        })
        .collect()
}

// Conver the given vertex config to internally used buffer layout
fn vertex_config_to_buffer_layout<'a>(
    vertex_config: &VertexConfig,
    attributes: Vec<&'a [wgpu::VertexAttribute]>,
) -> Vec<wgpu::VertexBufferLayout<'a>> {
    vertex_config
        .inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let attribute = &attributes[index];

            wgpu::VertexBufferLayout {
                array_stride: input.vertex_info().size() as u64,
                step_mode: input.step_mode(),
                attributes: attribute,
            }
        })
        .collect()
}

// Conver the statically typed color layout to the color target states needed for the fragment field
fn color_layout_to_color_target_state<C: ColorLayout>(
    blending: &Option<BlendConfig<C>>,
) -> Vec<Option<wgpu::ColorTargetState>> {
    // Convert the typed blending state array to a vector
    let vec: Option<Vec<Option<BlendState>>> = blending
        .as_ref()
        .map(|x| <C::BlendingArray as Into<Vec<Option<BlendState>>>>::into(*x));

    // Convert the typed layout targets to ColorTargetStates
    let targets = C::layout_info()
        .into_iter()
        .enumerate()
        .map(|(i, info)| {
            Some(wgpu::ColorTargetState {
                format: info.format(),
                blend: vec.as_ref().map(|vec| vec[i]).unwrap_or_default(),

                // FIXME: Let the user handle this
                write_mask: wgpu::ColorWrites::ALL,
            })
        })
        .collect::<Vec<_>>();
    targets
}

// Convert the depth and stencil config to the DepthStencilState
fn depth_stencil_config_to_state<DS: DepthStencilLayout>(
    depth_config: &Option<DepthConfig>,
    stencil_config: &Option<StencilConfig>,
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
    let depth_compare = depth_config
        .map(|dc| dc.compare)
        .unwrap_or(wgpu::CompareFunction::Never);
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
        PrimitiveConfig::Triangles {
            cull_face,
            wireframe: false,
            ..
        } => cull_face,
        _ => None,
    };

    let polygon_mode = match primitive_config {
        PrimitiveConfig::Triangles {
            wireframe: true, ..
        } => wgpu::PolygonMode::Line,
        _ => wgpu::PolygonMode::Fill,
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

impl<C: ColorLayout, DS: DepthStencilLayout> RenderPipeline<C, DS> {
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

    // Get the blend config used when creating this pipeline
    pub fn blend_config(&self) -> Option<&BlendConfig<C>> {
        self.blend_config.as_ref()
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
}
