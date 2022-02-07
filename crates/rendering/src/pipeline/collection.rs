use bitfield::SparseBitfield;
use ordered_vec::shareable::ShareableOrderedVec;

use crate::object::{PipelineObject, ObjectID};

// A collection that contains a specific type of pipeline object
pub struct Collection<T: PipelineObject> {
    inner: ShareableOrderedVec<T>,
}

impl<T: PipelineObject> Default for Collection<T> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<T: PipelineObject> Collection<T> {
    // Get 
    pub fn get(&self, id: ObjectID<T>) -> Option<&T> {
        self.inner.get(id.get()?)
    }
    // Get mutably
    pub fn get_mut(&mut self, id: ObjectID<T>) -> Option<&mut T> {
        self.inner.get_mut(id.get()?)
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)> {
        self.inner.iter_mut()
    }
    pub fn gen_id(&self) -> ObjectID<T> {
        ObjectID::new(self.inner.get_next_id_increment())
    }
    pub fn insert(&mut self, id: ObjectID<T>, val: T) -> Option<()> {
        self.inner.insert(id.get()?, val);
        Some(())
    }
    pub fn remove(&mut self, id: ObjectID<T>) -> Option<T> {
        self.inner.remove(id.get()?)
    }
}

// A tracked collection that detects whenever we mutated an element
pub struct TrackedCollection<T: PipelineObject> {
    inner: ShareableOrderedVec<T>,
    mutated: SparseBitfield,
}

impl<T: PipelineObject> Default for TrackedCollection<T> {
    fn default() -> Self {
        Self { inner: Default::default(), mutated: SparseBitfield::default() }
    }
}

impl<T: PipelineObject> TrackedCollection<T> {
    // Get 
    pub fn get(&self, id: ObjectID<T>) -> Option<&T> {
        self.inner.get(id.get()?)
    }
    // Get mutably
    pub fn get_mut(&mut self, id: ObjectID<T>) -> Option<&mut T> {
        self.inner.get_mut(id.get()?)
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)> {
        self.inner.iter_mut()
    }
    pub fn gen_id(&self) -> ObjectID<T> {
        ObjectID::new(self.inner.get_next_id_increment())
    }
    pub fn insert(&mut self, id: ObjectID<T>, val: T) -> Option<()> {
        self.inner.insert(id.get()?, val);
        Some(())
    }
    pub fn remove(&mut self, id: ObjectID<T>) -> Option<T> {
        self.inner.remove(id.get()?)
    }
}