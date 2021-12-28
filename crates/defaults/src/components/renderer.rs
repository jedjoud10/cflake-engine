use core::FrameID;
use rendering::{GPUObject, GPUObjectID, Material, ModelGPUObject, RendererFlags, RendererGPUObject};

// Wrapper
pub struct Renderer {
    pub internal_renderer: rendering::Renderer, // The internal renderer that we will pass to the render thread when creating this component
    pub matrix: veclib::Matrix4x4<f32>, // The model matrix of this renderer
    pub update_frame_id: FrameID,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            internal_renderer: rendering::Renderer::default(),
            matrix: veclib::Matrix4x4::default(),
            update_frame_id: FrameID::default(),
        }
    }
}

impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: GPUObjectID) -> Self {
        self.internal_renderer.model = Some(model);
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
    pub fn set_material(mut self, material: GPUObjectID) -> Self {
        self.internal_renderer.material = Some(material);
        self
    }
}

ecs::impl_component!(Renderer);
