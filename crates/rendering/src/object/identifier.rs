use super::PipelineObject;

use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

// This is a generic struct that hold an ID for a specific object stored in the multiple ShareableOrderedVecs in the pipeline
pub struct ObjectID<T>
where
    T: PipelineObject,
{
    id: Option<u64>,
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
impl<T: PipelineObject> std::hash::Hash for ObjectID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl<T: PipelineObject> PartialEq for ObjectID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T: PipelineObject> Eq for ObjectID<T> {}

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
    pub fn is_some(&self) -> bool {
        self.id.is_some()
    }
    // Try to get the internal ID
    pub fn get(&self) -> Option<u64> {
        self.id.clone()
    }
}

// Atomic counter that we will use to get the next reserved tracked ID
pub(crate) static RESERVED_TRACKED_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
// A tracking TaskID that we can use to check wether a specific task has executed or not
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReservedTrackedID(pub(crate) u64);

impl Default for ReservedTrackedID {
    // Reserve a special trakcing ID for ourselves
    // We should do this only once if we are running a tracked task multiple times
    fn default() -> Self {
        Self(RESERVED_TRACKED_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
