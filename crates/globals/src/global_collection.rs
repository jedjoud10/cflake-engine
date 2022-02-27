use crate::{
    error::GlobalError,
    global::{EnclosedGlobalComponent, Global},
};
use ahash::AHashMap;
use std::any::TypeId;

// A struct that will be stored in the world that will contain some globals
#[derive(Default)]
pub struct GlobalsCollection {
    pub(crate) globals: AHashMap<TypeId, EnclosedGlobalComponent>,
}

impl GlobalsCollection {
    // The reason why we can access global components but not normal components:
    // Since the normal components might be mutated in multiple threads, we cannot read from multiple components at the same time or we might cause UB.
    // However, global components will NEVER be mutated in multiple threads at the same time, so we can be 100% sure that we will never (hopefully) cause UB
    // Add a global component to the ECS manager
    pub fn add<U: Global + 'static>(&mut self, global: U) -> Result<(), GlobalError> {
        // UnsafeCell moment
        let boxed = Box::new(global);
        self.globals.insert(TypeId::of::<U>(), boxed);
        Ok(())
    }
    // Get a reference to a specific global component
    pub fn get<U: Global + 'static>(&self) -> Result<&U, GlobalError> {
        // First, we gotta check if this component was mutably borrowed
        // Kill me
        let hashmap = &self.globals;
        let boxed = hashmap
            .get(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let ptr = &*boxed.as_ref();
        let global = crate::registry::cast_global::<U>(ptr)?;
        Ok(global)
    }
    // Get a mutable reference to a specific global component
    pub fn get_mut<U: Global + 'static>(&mut self) -> Result<&mut U, GlobalError> {
        let hashmap = &mut self.globals;
        let boxed = hashmap
            .get_mut(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let ptr = &mut *boxed.as_mut();
        let global = crate::registry::cast_global_mut::<U>(ptr)?;
        Ok(global)
    }
}
