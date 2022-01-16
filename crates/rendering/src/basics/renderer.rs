use bitflags::bitflags;

use crate::{
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
};

use super::{material::Material, model::Model, Buildable};
// Yup
bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000001;
        const FADING_ANIMATION = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}

// A component that will be linked to entities that are renderable
pub struct Renderer {
    pub model: ObjectID<Model>,
    pub material: ObjectID<Material>,
    pub flags: RendererFlags,
    pub matrix: veclib::Matrix4x4<f32>,
}

impl PipelineObject for Renderer {}
ecs::impl_component!(Renderer);

impl Buildable for Renderer {
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.renderers.get_next_idx_increment();
        let id = ObjectID::new(id);
        (PipelineTask::CreateRenderer(ObjectBuildingTask::<Self>(self, id)), id)
    }
    fn pre_construct(mut self, pipeline: &Pipeline) -> Self {
        // We must fill out our model and material if they are empty
        let defaults = pipeline.defaults.as_ref().unwrap();
        if !self.model.valid() {
            self.model = defaults.model;
        }
        if !self.material.valid() {
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
    // Add a flag to our flags
    pub fn add_flag(mut self, flag: RendererFlags) -> Self {
        self.flags.insert(flag);
        self
    }
    // Remove a flag from our flags
    pub fn remove_flag(mut self, flag: RendererFlags) -> Self {
        self.flags.remove(flag);
        self
    }
}
