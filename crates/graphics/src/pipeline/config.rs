use crate::format::{Texel, Vertex, VertexInfo};
use crate::pass::ColorLayout;
use std::marker::PhantomData;
use wgpu::{ShaderLocation, VertexFormat, VertexStepMode};

/// How we read/write from depth attachments used by the material
#[derive(Clone, Copy, PartialEq)]
pub struct DepthConfig {
    /// Main depth compare function
    pub compare: CompareFunction,
    /// Should we write the depth?
    pub write_enabled: bool,
    /// Offset to use when doing depth tests
    pub depth_bias_constant: i32,
    /// Make slope affect depth bias (I think?)
    pub depth_bias_slope_scale: f32,
    /// Clamp depth bias
    pub depth_bias_clamp: f32,
}

/// WindingOrder of the triangles
pub type WindingOrder = wgpu::FrontFace;

/// Stencil config to use
pub type StencilConfig = wgpu::StencilState;

// Re-export comp-func and face
pub use wgpu::{CompareFunction, Face};

// Re-export blend state ops
pub use wgpu::{BlendComponent, BlendFactor, BlendOperation, BlendState};

/// How we will use color blending for each element
pub type BlendConfig<C: ColorLayout> = C::BlendingArray;

/// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveConfig {
    /// The pipeline will draw triangles onto the screen
    Triangles {
        /// Winding order of the triangles
        winding_order: WindingOrder,
        /// Optional face culling
        cull_face: Option<Face>,
        /// Should wireframe be used for rendering?
        wireframe: bool,
    },

    /// The pipeline will draw lines onto the screen
    Lines {
        width: f32,
    },

    /// The pipeline will draw points onto the screen
    Points,
}

/// This vertex config describes how vertices or more specifically, "vertex buffers" should be read
/// This maps the vertex buffer bindings directly to be usable by a render pass
#[derive(Default, Debug)]
pub struct VertexConfig {
    /// Inputs used for the vertex config
    pub inputs: Vec<VertexInputInfo>,
}

/// VertexInputInfo combines all the required info for a vertex input in one struct
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VertexInputInfo {
    info: VertexInfo,
    step_mode: VertexStepMode,
    shader_location: u32,
}

impl VertexInputInfo {
    /// Get the VertexInfo of the VertexInput
    pub fn vertex_info(&self) -> VertexInfo {
        self.info
    }

    /// Get the step mode of the VertexInput
    pub fn step_mode(&self) -> VertexStepMode {
        self.step_mode
    }

    /// Shader location of the vertex input
    pub fn shader_location(&self) -> u32 {
        self.shader_location
    }
}

/// Vertex input defines the vertex layout for a single buffer
pub trait VertexInput<V: Vertex> {
    /// Get the vertex info of the input
    fn vertex_info() -> VertexInfo {
        V::info()
    }

    /// Get the vertex step mode
    fn step_mode() -> VertexStepMode;

    /// Get the combined info (using a specific shader location)
    fn info(shader_location: u32) -> VertexInputInfo {
        VertexInputInfo {
            info: Self::vertex_info(),
            step_mode: Self::step_mode(),
            shader_location,
        }
    }
}

/// Defines a Vertex buffer/layout as being an input that
/// should be updated for every vertex drawn in the mesh
pub struct PerVertex<V: Vertex>(PhantomData<V>);
impl<V: Vertex> VertexInput<V> for PerVertex<V> {
    fn step_mode() -> VertexStepMode {
        VertexStepMode::Vertex
    }
}

/// Defines a Vertex buffer/layout as being an input
/// that should be updated for every instance drawn
pub struct PerInstance<V: Vertex>(PhantomData<V>);
impl<V: Vertex> VertexInput<V> for PerInstance<V> {
    fn step_mode() -> VertexStepMode {
        VertexStepMode::Instance
    }
}
