use super::{model::Model, Material};
use crate::{GPUObject, ModelGPUObject};
use assets::{Asset, AssetManager};

use bitflags::bitflags;
// Yup
bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}

// A component that will be linked to entities that are renderable
pub struct Renderer {
    pub index: usize, // The ID of this renderer in the pipeline 
    pub visible: bool,
    pub model: ModelGPUObject, // The model GPU of this renderer
    pub material: Material, // The CPU material of this renderer (We convert it to a GPU material when we add the renderer)
    pub flags: RendererFlags, // Flags
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            index: 0,
            visible: true,
            model: ModelGPUObject::default(),
            material: Material::default(),
            flags: RendererFlags::DEFAULT,
        }
    }
}

// Everything related to the creation of a renderer
impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: ModelGPUObject) -> Self {
        self.model = model;
        self
    }
    // Enable / disable the wireframe rendering for this entity
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.flags.remove(RendererFlags::WIREFRAME);
        }
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }
    // Set visible
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}
