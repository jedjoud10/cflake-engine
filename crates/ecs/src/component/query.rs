use crate::entity::EntityKey;
use super::LinkedComponents;
use ahash::AHashMap;
use std::{cell::{RefCell, RefMut, Ref}, rc::Rc};

// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
type LinkedComponentsMap = AHashMap<EntityKey, LinkedComponents>;
type MaybeRefCell = Option<Rc<RefCell<LinkedComponentsMap>>>;
pub struct ComponentQuery {
    // The actual components
    pub(crate) linked_components: MaybeRefCell,
}

impl ComponentQuery {
    // Count the number of linked components that we have
    pub fn get_entity_count(&self) -> usize {
        let len = self.linked_components.as_ref().map(|x| x.borrow().len());
        len.unwrap_or_default()
    }
    // Lock the component query, returning a ComponentQueryGuard that we can use to iterate over the components
    pub fn write(&mut self) -> RefMut<LinkedComponentsMap> {
        self.linked_components.as_ref().unwrap().borrow_mut()
    }
    pub fn read(&self) -> Ref<LinkedComponentsMap> {
        self.linked_components.as_ref().unwrap().borrow()
    }
}
