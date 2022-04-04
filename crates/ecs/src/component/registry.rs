use std::{
    any::{type_name, TypeId},
    collections::HashMap, cell::RefCell,
};
use crate::Mask;
use super::{Component, ComponentError};

// Internal registry that will be encapsulated in a RefCell
struct Internal {
    next: Mask,
    registered: HashMap<TypeId, Mask>,
}

impl Default for Internal {
    fn default() -> Self {
        Self { next: Mask::one(), registered: Default::default() }
    }
}

// A simple component registry that keeps track of the component masks
#[derive(Default)]
pub struct Registry {
    inner: RefCell<Internal>,
}

impl Registry {
    // Return the registered mask of the component
    #[inline(always)]
    pub fn mask<T: Component>(&self) -> Result<Mask, ComponentError> {
        let locked = self.inner.borrow();
        let id = TypeId::of::<T>();
        locked.registered.get(&id).ok_or(ComponentError::NotRegistered(Self::name::<T>())).cloned()
    }
    // Registers the component if it wasn't already registered
    #[inline(always)]
    pub fn register<T: Component>(&self) -> Mask {
        let mut locked = self.inner.borrow_mut();
        let id = TypeId::of::<T>();
        // If the component was already registered, no need to do anything
        if let Some(&bits) = locked.registered.get(&id) {
            return bits;
        }

        // Left bitshift
        let copy = locked.next; 
        locked.registered.insert(id, copy);
        locked.next = copy << 1;
        copy
    }
    // Get the name of a component
    #[inline(always)]
    pub fn name<T: Component>() -> &'static str {
        type_name::<T>()
    }
}