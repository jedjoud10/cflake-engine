use std::sync::{atomic::{AtomicU32, Ordering, AtomicU8}, Arc};
use crate::{basics::{transfer::{Transferable, Transfer}, Buildable}, object::{PipelineObject, ObjectID, PipelineTask, ObjectBuildingTask}};

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
    pub(crate) defaults: Vec<u32>,
    // When should we clear this atomic buffer?
    pub(crate) condition: ClearCondition, 
}

impl Default for AtomicGroup {
    fn default() -> Self {
        Self { 
            oid: 0,
            defaults: Vec::new(),
            condition: ClearCondition::DontClear
        }
    }
}

impl PipelineObject for AtomicGroup {
}

impl Buildable for AtomicGroup {
    fn construct_task(self, pipeline: &crate::pipeline::Pipeline) -> (crate::object::PipelineTask, crate::object::ObjectID<Self>) {
        // Create the ID
        let id = pipeline.atomics.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create a task and send it
        (PipelineTask::CreateAtomicGroup(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

impl AtomicGroup {
    // Create a new atomic counter with some predefined values
    pub fn new(vals: Vec<u32>) -> Self {
        Self {
            oid: 0,
            defaults: vals,
            condition: ClearCondition::DontClear
        }
    }
    // Set the clear condition
    pub fn set_clear_condition(mut self, condition: ClearCondition) -> Self {
        self.condition = condition;
        self
    }
}