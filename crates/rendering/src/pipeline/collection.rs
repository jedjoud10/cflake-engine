use ordered_vec::shareable::ShareableOrderedVec;
use crate::object::{ObjectID, PipelineObject};

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
    // Get and get mut
    pub fn get(&self, id: ObjectID<T>) -> Option<&T> {
        self.inner.get(id.get()?)
    }
    pub fn get_mut(&mut self, id: ObjectID<T>) -> Option<&mut T> {
        self.inner.get_mut(id.get()?)
    }
    // Iterators
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)> {
        self.inner.iter_mut()
    }
    // Generate a unique ID for a new object
    pub fn gen_id(&self) -> ObjectID<T> {
        ObjectID::new(self.inner.get_next_id_increment())
    }
    // Insert and remove
    pub fn insert(&mut self, id: ObjectID<T>, val: T) -> Option<()> {
        self.inner.insert(id.get()?, val);
        Some(())
    }
    pub fn remove(&mut self, id: ObjectID<T>) -> Option<T> {
        self.inner.remove(id.get()?)
    }
}