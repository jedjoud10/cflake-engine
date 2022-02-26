use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
};
use enum_as_inner::EnumAsInner;

// A light type
#[derive(EnumAsInner)]
pub enum LightSourceType {
    Directional { quat: veclib::Quaternion<f32> },
}

// Main struct that contains some shared data about a light source
pub struct LightSource {
    // Main
    pub _type: LightSourceType,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl PipelineObject for LightSource {
    // Reserve an ID for this light source
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.light_sources.gen_id()))
    }
    // Send this light source to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::LightSource(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::LightSource(Deconstruct::<Self>(id))
    }
    // Add the material to our ordered vec
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // We must return an error if we try to add multiple directional lights at the same time
        if pipeline.defaults.as_ref().unwrap().sun.is_some() && self._type.as_directional().is_some() {
            return None;
        }

        // We do not have a sun direction light yet, so if we are a light source of type "directional", we must add ourselves
        if self._type.as_directional().is_some() {
            pipeline.defaults.as_mut().unwrap().sun = id;
        }
        // Add the light source
        pipeline.light_sources.insert(id, self)?;
        Some(())
    }
    // Remove the light source from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.light_sources.remove(id)
    }
}

impl LightSource {
    // Create a new light source
    pub fn new(_type: LightSourceType) -> Self {
        Self {
            _type,
            color: veclib::Vector3::ONE,
            strength: 1.0,
        }
    }
    // Create this light source with a specififed strength
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength;
        self
    }
    // Create this light source with a specified color
    pub fn with_color(mut self, color: veclib::Vector3<f32>) -> Self {
        self.color = color;
        self
    }
}
