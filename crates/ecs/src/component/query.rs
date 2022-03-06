use crate::entity::EntityKey;
use super::LinkedComponents;
use ahash::AHashMap;
use std::{cell::{RefCell, RefMut, Ref}, rc::Rc, ops::{DerefMut, Deref}};

pub struct WriteGuard<'a, 'b> {
    inner: &'b mut RefMut<'a, LinkedComponentsMap>,
}

impl<'a, 'b> Deref for WriteGuard<'a, 'b> {
    type Target = LinkedComponentsMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, 'b> DerefMut for WriteGuard<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


pub struct ReadGuard<'a, 'b> {
    inner: &'b RefMut<'a, LinkedComponentsMap>,
}

impl<'a, 'b> Deref for ReadGuard<'a, 'b> {
    type Target = LinkedComponentsMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}



// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
type LinkedComponentsMap = AHashMap<EntityKey, LinkedComponents>;
pub struct ComponentQuery<'a> {
    // The actual components
    pub(crate) linked_components: Option<RefMut<'a, LinkedComponentsMap>>,
}

impl<'a> ComponentQuery<'a> {
    // Count the number of linked components that we have
    pub fn get_entity_count(&self) -> usize {
        let len = self.linked_components.as_ref().map(|x| x.len());
        len.unwrap_or_default()
    }
    // Lock the component query, returning a ComponentQueryGuard that we can use to iterate over the components
    pub fn write<'b>(&'b mut self) -> WriteGuard<'a, 'b> {
        let refmut = self.linked_components.as_mut().unwrap();
        WriteGuard { inner: refmut }
    }
    pub fn read<'b>(&'b self) -> ReadGuard<'a, 'b> {
        let refa = self.linked_components.as_ref().unwrap();
        ReadGuard { inner: refa }
    }
}
