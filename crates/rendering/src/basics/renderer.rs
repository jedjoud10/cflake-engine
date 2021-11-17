use super::{model::Model, model::ModelDataGPU, Material};
use crate::advanced::MultiMaterialRenderer;
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
    pub gpu_data: ModelDataGPU,
    pub model: Model,
    // This renderer can only have one material for now (TODO: Make a multi material system)
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
            gpu_data: ModelDataGPU::default(),
            model: Model::default(),
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
    // Load a model
    pub fn load_model(mut self, model_path: &str, asset_manager: &AssetManager) -> Option<Self> {
        self.model = Model::default().load_asset(model_path, &asset_manager.asset_cacher)?;
        return Some(self);
    }
    // Set a model
    pub fn set_model(mut self, model: Model) -> Self {
        self.model = model;
        return self;
    }
    // Enable / disable the wireframe rendering for this entity
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.flags.remove(RendererFlags::WIREFRAME);
        }
        return self;
    }
    // With a specific material
    pub fn set_material(mut self, material: Material) -> Self {
        self.material = material;
        return self;
    }
    // Set Multi Material Renderer
    pub fn set_multimat(mut self, multi_mat_renderer: MultiMaterialRenderer) -> Self {
        self.multi_material = Some(multi_mat_renderer);
        return self;
    }
    // Set visible
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl Renderer {
    // When we update the model and want to refresh it's OpenGL data
    pub fn refresh_model(&mut self) {
        self.gpu_data = self.model.refresh_gpu_data();
    }
    // Dispose of our model
    pub fn dispose_model(&mut self) {
        self.gpu_data.dispose();
    }
}
