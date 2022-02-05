use arrayvec::ArrayVec;

use crate::{object::{PipelineObject, ObjectID, ConstructionTask, Construct}, pipeline::Pipeline};

// The clear condition telling us when we should clear the atomic counter
#[derive(Clone)]
pub enum ClearCondition {
    BeforeShaderExecution,
    DontClear,
}

// A simple atomic counter that we can use inside OpenGL fragment and compute shaders, if possible
// This can store multiple atomic counters in a single buffer, thus making it a group
#[derive(Clone)]
pub struct AtomicGroup {
    // The OpenGL ID for the atomic counter buffer
    pub(crate) oid: u32,
    // Some predefined values that we can set before we execute the shader
    // This also stores the number of valid atomics that we have
    pub(crate) defaults: ArrayVec<u32, 4>,
    // When should we clear this atomic buffer?
    pub(crate) condition: ClearCondition,
}

impl Default for AtomicGroup {
    fn default() -> Self {
        let mut arrayvec = ArrayVec::<u32, 4>::new();
        arrayvec.push(0);
        Self {
            oid: 0,
            defaults: arrayvec,
            condition: ClearCondition::DontClear,
        }
    }
}

impl PipelineObject for AtomicGroup {
    // Reserve an ID for this atomic group
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.atomics.get_next_id_increment())))
    }
    // Send this atomic group to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::AtomicGroup(Construct::<Self>(self, id))
    }
    // Add the atomic group to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the atomic group
        pipeline.atomics.insert(id.get()?, self);
        Some(())
    }
    // Remove the atomic group from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.atomics.remove(id.get()?)
    }
}

impl AtomicGroup {
    // Create a new atomic counter with some predefined values
    pub fn new(vals: &[u32]) -> Option<Self> {
        let mut arrayvec = ArrayVec::<u32, 4>::new();
        arrayvec.try_extend_from_slice(vals).ok()?;
        Some(Self {
            oid: 0,
            defaults: arrayvec,
            condition: ClearCondition::DontClear,
        })
    }
    // Set the clear condition
    pub fn set_clear_condition(mut self, condition: ClearCondition) -> Self {
        self.condition = condition;
        self
    }
}
