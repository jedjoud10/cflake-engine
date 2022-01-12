use super::{PipelineObject};

use std::{
    marker::PhantomData,
};

// This is a generic struct that hold an ID for a specific object stored in the multiple ShareableOrderedVecs in the pipeline
pub struct ObjectID<T>
where
    T: PipelineObject,
{
    pub(crate) index: Option<usize>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: PipelineObject> Clone for ObjectID<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _phantom: self._phantom,
        }
    }
}

impl<T: PipelineObject> Copy for ObjectID<T> {}

impl<T> Default for ObjectID<T>
where
    T: PipelineObject,
{
    fn default() -> Self {
        Self {
            index: None,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> ObjectID<T>
where
    T: PipelineObject,
{
    // Create a new object ID using an actual index
    pub fn new(index: usize) -> Self {
        Self {
            index: Some(index),
            _phantom: PhantomData::default(),
        }
    }
    // Check if this ID is even valid LOCALLY
    pub fn valid(&self) -> bool {
        self.index.is_some()
    }
}

// This is an ID for each Task that we dispatch to the render thread.
// We can use this to detect whenever said task has completed
#[derive(Clone, Copy)]
pub struct TaskID {
    pub(crate) index: usize,
}

impl TaskID {
    // Create a new task ID using an actual index
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
