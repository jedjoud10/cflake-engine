use std::marker::PhantomData;
use wgpu::{VertexFormat, VertexStepMode, ShaderLocation};
pub use wgpu::{FrontFace, Face};
use crate::{VertexInfo, Vertex};

// TODO: Write binding config for use with WGPU
pub struct BindingConfig {
}

// Blend config for a single color attachment
#[derive(Clone, Copy)]
pub struct AttachmentBlendConfig {
}


// How we deal with blending for the color attachments
#[derive(Clone, Copy)]
pub struct BlendConfig {
}

// How we read/write from depth attachments used by the material
#[derive(Clone, Copy, PartialEq)]
pub struct DepthConfig {
    pub compare: Compare,
    pub write_enabled: bool,
    pub depth_bias_constant: i32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
}

pub type Compare = wgpu::CompareFunction;
pub type StencilConfig = wgpu::StencilState;

// Depicts the exact primitives we will use to draw the VAOs
#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveConfig {
    // The pipeline will draw triangles onto the screen
    Triangles {
        winding_order: FrontFace,
        cull_face: Option<Face>,
        wireframe: bool,
    },

    // The pipeline will draw lines onto the screen
    Lines {
        width: f32,
    },

    // The pipeline will draw points onto the screen
    Points,
}

// This vertex config describes how vertices or more specifically, "vertex buffers" should be read
// This maps the vertex buffer bindings directly to be usable by a render pass 
pub struct VertexConfig {
    pub inputs: Vec<VertexInputInfo>,
}

// VertexInputInfo combines all the required info for a vertex input in one struct
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VertexInputInfo {
    location: ShaderLocation,
    info: VertexInfo,
    step_mode: VertexStepMode,
}

impl VertexInputInfo {
    // Get the shader location of the VertexInput
    pub fn location(&self) -> ShaderLocation {
        self.location
    }

    // Get the VertexInfo of the VertexInput
    pub fn vertex_info(&self) -> VertexInfo {
        self.info
    }

    // Get the step mode of the VertexInput
    pub fn step_mode(&self) -> VertexStepMode {
        self.step_mode
    }
}

// Vertex input defines the vertex layout for a single buffer
// TODO: Implement vertex interlacing
pub trait VertexInput<V: Vertex> {
    // Create a new vertex input (layout) using the appropriate shader location
    fn new(location: ShaderLocation) -> Self where Self: Sized;

    // Get the shader location of the vertex input
    fn location(&self) -> ShaderLocation;

    // Get the vertex info of the input
    fn vertex_info(&self) -> VertexInfo {
        V::info()
    }

    // Get the vertex step mode
    fn step_mode(&self) -> VertexStepMode;

    // Get the combined info
    fn info(&self) -> VertexInputInfo {
        VertexInputInfo {
            location: self.location(),
            info: self.vertex_info(),
            step_mode: self.step_mode(),
        }
    }
}

// Defines a Vertex buffer/layout as being an input that
// should be updated for every vertex drawn in the mesh
pub struct PerVertex<V: Vertex>(PhantomData<V>, ShaderLocation);
impl<V: Vertex> VertexInput<V> for PerVertex<V> {
    fn new(location: ShaderLocation) -> Self {
        Self(PhantomData, location)
    }

    fn location(&self) -> ShaderLocation {
        self.1
    }

    fn step_mode(&self) -> VertexStepMode {
        VertexStepMode::Vertex
    }
}

// Defines a Vertex buffer/layout as being an input
// that should be updated for every instance drawn
pub struct PerInstance<V: Vertex>(PhantomData<V>, ShaderLocation);
impl<V: Vertex> VertexInput<V> for PerInstance<V> {
    fn new(location: ShaderLocation) -> Self {
        Self(PhantomData, location)
    }

    fn location(&self) -> ShaderLocation {
        self.1
    }

    fn step_mode(&self) -> VertexStepMode {
        VertexStepMode::Instance
    }
}