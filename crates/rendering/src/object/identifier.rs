use std::{marker::PhantomData, sync::{Arc, atomic::AtomicPtr}};
use crate::{Pipeline, Buildable};
use super::{PipelineObject, PipelineTaskStatus};

// This is a generic struct that hold an ID for a specific object stored in the multiple ShareableOrderedVecs in the pipeline
#[derive(Default)]
pub struct ObjectID<T>
    where T: PipelineObject + Buildable
{
    pub(crate) index: usize,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> ObjectID<T>
    where T: PipelineObject + Buildable
{
    // Create a new object ID using an actual index
    pub fn new(index: usize) -> Self {
        Self {
            index, _phantom: PhantomData::default()
        }
    }
}
// We must implement watchable separately :(


// This is an ID for each Task that we dispatch to the render thread.
// We can use this to detect whenever said task has completed
pub struct TaskID {
    pub(crate) index: usize,
}

impl TaskID {
    // Create a new task ID using an actual index
    pub fn new(index: usize) -> Self {
        Self {
            index
        }
    }
}

impl others::Watchable<Pipeline> for TaskID {
    fn get_uid(&self) -> usize {
        self.index
    }

    fn is_valid(&self, context: &Pipeline) -> bool {
        // Try to get the task and check if it is valid
        if let Some(status) =  context.task_statuses.get(self.index) {
            if let PipelineTaskStatus::Finished = *status {
                true
            } else { false }
        } else { false }
    }
}