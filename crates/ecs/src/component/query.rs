use super::LinkedComponents;
use crate::entity::EntityKey;
use ahash::AHashMap;
use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

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
    pub(crate) linked_components: &'a mut LinkedComponentsMap,
}

impl<'a> Deref for ComponentQuery<'a> {
    type Target = LinkedComponentsMap;

    fn deref(&self) -> &Self::Target {
        self.linked_components
    }
}

impl<'a> DerefMut for ComponentQuery<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.linked_components
    }
}
