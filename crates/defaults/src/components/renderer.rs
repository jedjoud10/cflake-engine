use main::ecs::component::Component;
use main::rendering::basics::material::Material;
use main::rendering::basics::model::Model;
use main::rendering::basics::renderer::RendererFlags;
use main::rendering::basics::uniforms::SetUniformsCallback;
use main::rendering::pipeline::{Pipeline, pipec};
use main::rendering::{self, object::ObjectID};
type GPURenderer = rendering::basics::renderer::Renderer; 
// An Renderer component
#[derive(Component)]
pub struct Renderer {
    // The CPU renderer that we will store until we send the construction task
    pub(crate) inner: Option<GPURenderer>,

    // The returned Object ID for our Renderer that is stored on the GPU Pipeline
    pub(crate) id: ObjectID<rendering::basics::renderer::Renderer>,
}

impl Renderer {
    // Create a new renderer component
    pub fn new(flags: RendererFlags) -> Self {
        Self {
            inner: Some(GPURenderer::new(flags)),
            id: ObjectID::default(),
        }
    }
    // Set a model
    pub fn with_model(mut self, model: ObjectID<Model>) -> Self {
        self.inner.as_mut().unwrap().model = model;
        self
    }
    // With a specific material
    pub fn with_material(mut self, material: ObjectID<Material>) -> Self {
        self.inner.as_mut().unwrap().material = material;
        self
    }
    // Set the model matrix for this renderer
    pub fn with_matrix(mut self, matrix: veclib::Matrix4x4<f32>) -> Self {
        self.inner.as_mut().unwrap().matrix = matrix;
        self
    }
    // Set a uniform callback for this renderer (This is not ideal, but it's better than the last method)
    pub fn with_uniform(mut self, callback: SetUniformsCallback) -> Self {
        self.inner.as_mut().unwrap().uniforms = callback;
        self
    }
    // Update this renderer's model on the GPU, destroying the old one
    pub fn update_model(&mut self, pipeline: &Pipeline, new_model: ObjectID<Model>) {
        // Get the GPU renderer
        let renderer = pipeline.renderers.get(self.id).unwrap();
        // Destroy the model
        pipec::deconstruct(pipeline, renderer.model);
        // Make a callback to set our new model
        let renderer_id = self.id;
        pipec::update_callback(pipeline, move |pipeline, _scene_renderer| {
            let renderer = pipeline.renderers.get_mut(renderer_id).unwrap();
            renderer.model = new_model;
        })
    }
}
