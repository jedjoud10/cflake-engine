use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
};
use super::{material::Material, model::Model, uniforms::ShaderUniformsGroup};
use bitflags::bitflags;
bitflags! {
    pub struct RendererFlags: u8 {
        const VISIBLE = 0b00000001;
        const SHADOW_CASTER = 0b00000010;
        const SHOULD_DELETE_MODEL = 0b00000100;
        const DEFAULT = Self::VISIBLE.bits | Self::SHADOW_CASTER.bits | Self::SHOULD_DELETE_MODEL.bits;
    }
}

// A component that will be linked to entities that are renderable
pub struct Renderer {
    // Rendering
    pub model: ObjectID<Model>,
    pub material: ObjectID<Material>,
    pub matrix: veclib::Matrix4x4<f32>,
    pub flags: RendererFlags,

    // Some renderer specific uniforms that may override the material uniforms when rendering
    pub uniforms: Option<ShaderUniformsGroup>,

}

impl Renderer {
    // Create a new renderer with default settings
    pub fn new(flags: RendererFlags) -> Self {
        Self {
            model: Default::default(),
            material: Default::default(),
            matrix: Default::default(),
            uniforms: Default::default(),
            flags,
        }
    }
}
impl PipelineObject for Renderer {
    // Reserve an ID for this renderer
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.renderers.get_next_id_increment())))
    }
    // Send this rendererer to the pipeline for construction
    fn send(self, _pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Renderer(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(_pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Renderer(Deconstruct::<Self>(id))
    }
    // Add the renderer to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Get the renderer data, if it does not exist then use the default renderer data
        let defaults = pipeline.defaults.as_ref()?;
        // Make sure we have valid fields
        if !self.model.is_some() {
            self.model = defaults.model;
        }
        if !self.material.is_some() {
            self.material = defaults.material;
        }
        // Add the renderer
        pipeline.renderers.insert(id.get()?, self);
        Some(())
    }
    // Delete the renderer from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let me = pipeline.renderers.remove(id.get()?)?;
        // Also remove the model if we want to
        if me.flags.contains(RendererFlags::SHOULD_DELETE_MODEL) {
            let _removed_model = Model::delete(pipeline, me.model)?;
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
    // Update a renderer's matrix
    pub fn update_matrix(&mut self, matrix: veclib::Matrix4x4<f32>) {
        self.matrix = matrix;
    }
}
