use bitflags::bitflags;

use crate::{object::{ObjectID, PipelineObject, ObjectBuildingTask, PipelineTask}, Material, Model, Buildable};
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
    pub model: Option<ObjectID<Model>>, 
    pub material: Option<ObjectID<Material>>, 
    pub flags: RendererFlags,
}

impl PipelineObject for Renderer {}

impl Buildable for Renderer {
    fn construct(self, pipeline: &crate::Pipeline) -> ObjectID<Self> {
        // Create the ID
        let id = pipeline.renderers.get_next_idx_increment();
        let id = ObjectID::new(id);
        crate::pipec::task(PipelineTask::CreateRenderer(ObjectBuildingTask::<Self>(self, id)), pipeline);
        id
    }
}

// Everything related to the creation of a renderer
impl Renderer {
    // Set a model
    pub fn set_model(mut self, model: ObjectID<Model>) -> Self {
        self.model = Some(model);
        self
    }
    // With a specific material
    pub fn set_material(mut self, material: ObjectID<Material>) -> Self {
        self.material = Some(material);
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
