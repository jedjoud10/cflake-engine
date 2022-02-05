use crate::object::{ObjectID, PipelineObject, ConstructionTask, Construct};

use super::{model::Model, material::Material, uniforms::ShaderUniformsGroup};

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

impl PipelineObject for Renderer {
    // Reserve an ID for this renderer
    fn reserve(self, pipeline: &crate::pipeline::Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.renderers.get_next_id_increment())))
    }
    // Send this rendererer to the pipeline for construction
    fn send(self, pipeline: &crate::pipeline::Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Renderer(Construct::<Self>(self, id))
    }
    // Add the renderer to our ordered vec
    fn add(self, pipeline: &mut crate::pipeline::Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Get the renderer data, if it does not exist then use the default renderer data
        let defaults = pipeline.defaults.as_ref()?;
        let _material_id = pipeline.get_material(defaults.material)?;
        let _model_id = pipeline.get_model(defaults.model)?;
        // Make sure we have valid fields
        if !self.model.is_some() { self.model = defaults.model; }
        if !self.material.is_some() { self.material = defaults.material; }
        // Add the renderer
        pipeline.renderers.insert(id.get()?, self);
        Some(())
    }
    // Delete the renderer from the pipeline
    fn delete(pipeline: &mut crate::pipeline::Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let me = pipeline.renderers.remove(id.get()?)?;
        // Also remove the model if we want to
        if me.delete_model {
            let removed_model = Model::delete(pipeline, me.model)?;
        }
        Some(me)
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
}