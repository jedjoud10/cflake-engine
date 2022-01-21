use super::PipelineObject;

use std::{fmt::Debug, marker::PhantomData};

// This is a generic struct that hold an ID for a specific object stored in the multiple ShareableOrderedVecs in the pipeline
pub struct ObjectID<T>
where
    T: PipelineObject,
{
    pub(crate) id: Option<u64>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: PipelineObject> Clone for ObjectID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: self._phantom,
        }
    }
}

impl<T: PipelineObject> Copy for ObjectID<T> {}

impl<T: PipelineObject> Debug for ObjectID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectID").field("id", &self.id).finish()
    }
}

impl<T> Default for ObjectID<T>
where
    T: PipelineObject,
{
    fn default() -> Self {
        Self {
            id: None,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T> ObjectID<T>
where
    T: PipelineObject,
{
    // Create a new object ID using an actual index
    pub fn new(id: u64) -> Self {
        Self {
            id: Some(id),
            _phantom: PhantomData::default(),
        }
    }
    // Check if this ID is even valid LOCALLY
    pub fn valid(&self) -> bool {
        self.id.is_some()
    }
}

// This is an ID for each Task that we dispatch to the render thread.
// We can use this to detect whenever said task has completed
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskID {
    pub(crate) id: u64,
}

impl TaskID {
    // Create a new task ID using an actual index
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}
