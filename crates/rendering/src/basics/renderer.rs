use super::Material;
use crate::{GPUObjectID, MaterialGPUObject, ModelGPUObject, ShaderUniformsGroup};

use bitflags::bitflags;
// Yup
bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000001;
        const FADING_ANIMATION = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}

// A component that will be linked to entities that are renderable
#[derive(Clone)]
pub struct Renderer {
    pub index: Option<GPUObjectID>,    // The ID of this renderer in the pipeline
    pub model: Option<GPUObjectID>,    // The model GPU of this renderer
    pub material: Option<GPUObjectID>, // The GPU material of this renderer
    pub uniforms: Option<ShaderUniformsGroup>,
    pub flags: RendererFlags, // Flags
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            index: None,
            model: None,
            material: None,
            uniforms: None,
            flags: RendererFlags::DEFAULT,
        }
    }
}

// Everything related to the creation of a renderer
impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: GPUObjectID) -> Self {
        self.model = Some(model);
        self
    }
    // Enable / disable the wireframe rendering
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.flags.remove(RendererFlags::WIREFRAME);
        }
        self
    }
    // Enable / disable fading animation
    pub fn set_fading_animation(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::FADING_ANIMATION);
        } else {
            self.flags.remove(RendererFlags::FADING_ANIMATION);
        }
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: GPUObjectID) -> Self {
        self.material = Some(material);
        self
    }
    // Set a specific shader uniform for this renderer
    pub fn set_shader_uniforms(mut self, shader_uniforms: ShaderUniformsGroup) -> Self {
        self.uniforms = Some(shader_uniforms);
        self
    }
}
