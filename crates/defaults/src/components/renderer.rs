use core::FrameID;
use rendering::{GPUObject, GPUObjectID, Material, ModelGPUObject, RendererFlags, RendererGPUObject, ShaderUniformsGroup};

// Wrapper
pub struct Renderer {
    pub internal_renderer: rendering::Renderer, // The internal renderer that we will pass to the render thread when creating this component
    pub matrix: veclib::Matrix4x4<f32>,         // The model matrix of this renderer
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
        self.internal_renderer = self.internal_renderer.set_model(model);
        self
    }
    // Enable / disable the wireframe rendering
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        self.internal_renderer = self.internal_renderer.set_wireframe(enabled);
        self
    }
    // Enable / disable the fading animation
    pub fn set_fading_animation(mut self, enabled: bool) -> Self {
        self.internal_renderer = self.internal_renderer.set_fading_animation(enabled);
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: GPUObjectID) -> Self {
        self.internal_renderer = self.internal_renderer.set_material(material);
        self
    }
    // Set a specific shader uniform for this renderer
    pub fn set_shader_uniforms(mut self, shader_uniforms: ShaderUniformsGroup) -> Self {
        self.internal_renderer = self.internal_renderer.set_shader_uniforms(shader_uniforms);
        self
    }
}

ecs::impl_component!(Renderer);
