use std::any::TypeId;

use wgpu::TextureFormat;
use crate::{ChannelsType, ElementType};

// Untyped texel info that does not contain typed information about the texel nor base types
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TexelInfo {
    pub(crate) bytes_per_channel: u32,
    pub(crate) element: ElementType,
    pub(crate) channels: ChannelsType,
    pub(crate) format: TextureFormat,
}

impl TexelInfo {
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

    // Type of channels (either R, RG, RGBA, BGRA, Depth, Stencil)
    pub fn channels(&self) -> ChannelsType {
        self.channels
    }

    // Compile time WGPU format
    pub fn format(&self) -> TextureFormat {
        self.format
    }
}