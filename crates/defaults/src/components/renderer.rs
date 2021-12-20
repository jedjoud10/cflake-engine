use ecs::{Component, ComponentID, ComponentInternal};
use rendering::{GPUObject, Material, ModelGPUObject, RendererFlags};

// Wrapper
pub struct Renderer {
    pub internal_renderer: rendering::Renderer,
}

impl Default for Renderer {
    fn default() -> Self {
        Self { internal_renderer: Default::default() }
    }
}

impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: ModelGPUObject) -> Self {
        self.internal_renderer.model = model;
        self
    }
    // Enable / disable the wireframe rendering for this entity
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.internal_renderer.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.internal_renderer.flags.remove(RendererFlags::WIREFRAME);
        }
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: Material) -> Self {
        self.internal_renderer.material = material;
        self
    }
    // Set visible
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.internal_renderer.visible = visible;
        self
    }
}

ecs::impl_component!(Renderer);
