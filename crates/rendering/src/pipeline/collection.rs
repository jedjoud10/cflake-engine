use crate::object::{ObjectID, PipelineObject};
use bitfield::SparseBitfield;
use ordered_vec::shareable::ShareableOrderedVec;

// A collection that contains a specific type of pipeline object
pub struct Collection<T: PipelineObject> {
    inner: ShareableOrderedVec<T>,
    mutated: SparseBitfield,
}

impl<T: PipelineObject> Default for Collection<T> {
    fn default() -> Self {
        Self { inner: Default::default(), mutated: Default::default() }
    }
}

impl<T: PipelineObject> Collection<T> {
    // Get and get mut
    pub fn get(&self, id: ObjectID<T>) -> Option<&T> {
        self.inner.get(id.get()?)
    }
    pub fn get_mut(&mut self, id: ObjectID<T>) -> Option<&mut T> {
        let id = id.get()?;
        // Update the bitfield if we want to
        if T::UPDATE { 
            let pair = ordered_vec::utils::from_id(id);
            self.mutated.set(pair.index as usize, true)
        }
        self.inner.get_mut(id)
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
        let id = id.get()?;
        self.inner.insert(id, val);
        // Update the bitfield if we want to
        if T::UPDATE { 
            let pair = ordered_vec::utils::from_id(id);
            self.mutated.set(pair.index as usize, true)
        }
        Some(())
    }
    pub fn remove(&mut self, id: ObjectID<T>) -> Option<T> {
        self.inner.remove(id.get()?)
    }
    // Diffs
    pub(crate) fn clear_diffs(&mut self) { self.mutated.clear() }
    #[allow(dead_code)]
    pub(crate) fn was_mutated(&self, id: ObjectID<T>) -> bool { 
        let id = if let Some(x) = id.get() { x } else { return false };
        let pair = ordered_vec::utils::from_id(id);
        self.mutated.get(pair.index as usize)
    }
}
