use wgpu::VertexFormat;
use crate::{ElementType, VectorChannels};

// Untyped texel info that does not contain typed information about the vertex nor base types
pub struct VertexInfo {
    pub(crate) bytes_per_channel: u32,
    pub(crate) element: ElementType,
    pub(crate) channels: VectorChannels,
    pub(crate) format: VertexFormat,
}

impl VertexInfo {
    // Number of bytes in total
    pub fn size(&self) -> u32 {
        self.bytes_per_channel * self.channels.count()
    }

    // Number of bytes per channel
    pub fn bytes_per_channel(&self) -> u32 {
        self.bytes_per_channel
    }

    // Untyped representation of the underlying element
    pub fn element(&self) -> ElementType {
        self.element
    }
    
    // Type of channels (either X, XY, XYZ, XYZW)
    pub fn channels(&self) -> VectorChannels {
        self.channels
    }
    
    // Compile time WGPU format
    pub fn format(&self) -> VertexFormat {
        self.format
    }
}