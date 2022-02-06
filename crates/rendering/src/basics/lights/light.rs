use crate::{object::{PipelineObject, ObjectID, ConstructionTask, Construct, DeconstructionTask, Deconstruct}, pipeline::Pipeline};

use super::Directional;

// A light type
pub enum LightSourceType {
    Directional(Directional),
}

// Main struct that contains some shared data about a light source
pub struct LightSource {
    inner: LightSourceType,
    pub strength: f32,
}

impl PipelineObject for LightSource {
    // Reserve an ID for this light source
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.light_sources.get_next_id_increment())))
    }
    // Send this light source to the pipeline for construction
    fn send(self, _pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::LightSource(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(_pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::LightSource(Deconstruct::<Self>(id))
    }
    // Add the material to our ordered vec
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the light source
        pipeline.light_sources.insert(id.get()?, self);
        Some(())
    }
    // Remove the light source from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.light_sources.remove(id.get()?)
    }
}

impl LightSource {
    // Create a new light source
    pub fn new(_type: LightSourceType) -> Self {
        Self {
            inner: _type,
            strength: 1.0
        }
    }
    // With strength
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength;
        self
    }
    // Get the directional light data if our inner light type is directional
    pub fn get_directional(&self) -> Option<&Directional> {
        if let LightSourceType::Directional(d) = &self.inner { 
            return Some(d);
        } { return None; }
    }
    // Get the directional light data if our inner light type is directional mutably
    pub fn get_directional_mut(&mut self) -> Option<&mut Directional> {
        if let LightSourceType::Directional(d) = &mut self.inner { 
            return Some(d);
        } { return None; }
    }
}