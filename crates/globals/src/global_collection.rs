use std::{any::TypeId, cell::UnsafeCell};

use ahash::AHashMap;

use crate::{
    error::GlobalError,
    global::{EnclosedGlobalComponent, Global, GlobalReadGuard, GlobalWriteGuard},
};

// A struct that will be stored in the world that will contain some globals
#[derive(Default)]
pub struct GlobalCollection {
    pub(crate) globals: AHashMap<TypeId, UnsafeCell<EnclosedGlobalComponent>>,
}

impl GlobalCollection {
    // The reason why we can access global components but not normal components:
    // Since the normal components might be mutated in multiple threads, we cannot read from multiple components at the same time or we might cause UB.
    // However, global components will NEVER be mutated in multiple threads at the same time, so we can be 100% sure that we will never (hopefully) cause UB
    // Add a global component to the ECS manager
    pub fn add_global<U: Global + 'static>(&mut self, sc: U) -> Result<(), GlobalError> {
        // UnsafeCell moment
        let boxed = Box::new(sc);
        self.globals.insert(TypeId::of::<U>(), UnsafeCell::new(boxed));
        Ok(())
    }
    // Get a reference to a specific global component
    pub fn get_global<'a, U: Global + 'static>(&'a self) -> Result<GlobalReadGuard<'a, U>, GlobalError> {
        // First, we gotta check if this component was mutably borrowed
        // Kill me
        let hashmap = &self.globals;
        let boxed = hashmap
            .get(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let ptr = unsafe { &*boxed.get() }.as_ref();
        let global = crate::registry::cast_global::<U>(ptr)?;
        Ok(GlobalReadGuard::new(global))
    }
    // Get a mutable reference to a specific global component
    pub fn get_global_mut<'a, U: Global + 'static>(&'a mut self) -> Result<GlobalWriteGuard<'a, U>, GlobalError> {
        let hashmap = &mut self.globals;
        let boxed = hashmap
            .get_mut(&TypeId::of::<U>())
            .ok_or_else(|| GlobalError::new("Global component could not be fetched!".to_string()))?;
        // Magic
        let ptr = unsafe { &mut *boxed.get() }.as_mut();
        let global = crate::registry::cast_global_mut::<U>(ptr)?;
        Ok(GlobalWriteGuard::new(global))
    }
}
