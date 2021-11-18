use super::{model::Model, model::ModelDataGPU, Material};
use crate::{GPUObject, advanced::MultiMaterialRenderer};
use assets::{Asset, AssetManager};
use ecs::{Component, ComponentID, ComponentInternal};

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
    pub visible: bool,
    pub model: GPUObject,
    pub material: Material,
    // Flags
    pub flags: RendererFlags,
    // Multi material support
    pub multi_material: Option<MultiMaterialRenderer>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            visible: true,
            model: GPUObject::None,
            material: Material::default(),
            flags: RendererFlags::DEFAULT,
            multi_material: None,
        }
    }
}

// Main traits implemented
ecs::impl_component!(Renderer);

// Everything related to the creation of a renderer
impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: GPUObject) -> Self {
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
    // Set Multi Material Renderer
    pub fn set_multimat(mut self, multi_mat_renderer: MultiMaterialRenderer) -> Self {
        self.multi_material = Some(multi_mat_renderer);
        self
    }
    // Set visible
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}