use crate::{
    error::GlobalError,
    global::{BoxedGlobal, Global},
};
use ahash::AHashMap;
use std::any::TypeId;

// A struct that will be stored in the world that will contain some globals
#[derive(Default)]
pub struct GlobalsSet {
    pub(crate) globals: AHashMap<TypeId, BoxedGlobal>,
}

impl GlobalsSet {
    // The reason why we can access global components but not normal components:
    // Since the normal components might be mutated in multiple threads, we cannot read from multiple components at the same time or we might cause UB.
    // However, global components will NEVER be mutated in multiple threads at the same time, so we can be 100% sure that we will never (hopefully) cause UB
    // Add a global component to the ECS manager
    pub fn insert<U: Global + 'static>(&mut self, global: U) -> Result<(), GlobalError> {
        // UnsafeCell moment
        let boxed = Box::new(global);
        self.globals.insert(TypeId::of::<U>(), boxed);
        Ok(())
    }
    // Get a reference to a specific global component
    pub fn get<U: Global + 'static>(&self) -> Result<&U, GlobalError> {
        let hashmap = &self.globals;
        let boxed = hashmap
            .get(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let any = &*boxed.as_ref().as_any();
        let global = any.downcast_ref::<U>().ok_or_else(|| GlobalError::new("Could not cast global!".to_string()))?;
        Ok(global)
    }
    // Get a mutable reference to a specific global component
    pub fn get_mut<U: Global + 'static>(&mut self) -> Result<&mut U, GlobalError> {
        let hashmap = &mut self.globals;
        let boxed = hashmap
            .get_mut(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let any = &mut *boxed.as_mut().as_any_mut();
        let global = any.downcast_mut::<U>().ok_or_else(|| GlobalError::new("Could not cast global!".to_string()))?;
        Ok(global)
    }
}
