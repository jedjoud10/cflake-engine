use crate::{
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
};

use super::{material::Material, model::Model, uniforms::ShaderUniformsGroup, Buildable};

// A component that will be linked to entities that are renderable
pub struct Renderer {
    // Rendering
    pub model: ObjectID<Model>,
    pub material: ObjectID<Material>,
    pub matrix: veclib::Matrix4x4<f32>,

    // Some renderer specific uniforms that may override the material uniforms when rendering
    pub uniforms: Option<ShaderUniformsGroup>,

    // Should we dispose the model when this renderer gets destroyed?
    pub delete_model: bool,
}

impl Renderer {
    // Create a new renderer with default settings
    pub fn new(delete_model: bool) -> Self {
        Self {
            model: Default::default(),
            material: Default::default(),
            matrix: Default::default(),
            uniforms: Default::default(),
            delete_model,
        }
    }
}

impl PipelineObject for Renderer {}

impl Buildable for Renderer {
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.renderers.get_next_id_increment();
        let id = ObjectID::new(id);
        (PipelineTask::CreateRenderer(ObjectBuildingTask::<Self>(self, id)), id)
    }
    fn pre_construct(mut self, pipeline: &Pipeline) -> Self {
        // We must fill out our model and material if they are empty
        let defaults = pipeline.defaults.as_ref().unwrap();
        if !self.model.is_some() {
            self.model = defaults.model;
        }
        if !self.material.is_some() {
            self.material = defaults.material;
        }
        self
    }
}

// Everything related to the creation of a renderer
impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: ObjectID<Model>) -> Self {
        self.model = model;
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: ObjectID<Material>) -> Self {
        self.material = material;
        self
    }
    // Set the model matrix for this renderer
    pub fn set_matrix(mut self, matrix: veclib::Matrix4x4<f32>) -> Self {
        self.matrix = matrix;
        self
    }
    // Update our uniforms
    pub fn update_uniforms(&mut self, uniforms: ShaderUniformsGroup) {
        self.uniforms = Some(uniforms);
    }
    // Set
}
