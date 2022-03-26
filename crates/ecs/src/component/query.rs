use crate::{
    archetype::{ArchetypeId, ArchetypeSet, ComponentsHashMap},
    manager::EcsManager,
};
use getset::CopyGetters;
use parking_lot::RwLock;
use std::{cell::UnsafeCell, sync::Arc, marker::PhantomData};

use super::{
    component_err, unlinked_err, Component, QueryError, ADDED_STATE, MUTATED_STATE, REMOVED_STATE,
};

// Component deltas (each component has a state of either [Mutated, Added, PendingForDeletion])
#[derive(CopyGetters)]
pub struct ComponentDeltas {
    #[getset(get_copy = "pub")]
    state: u8,
}

impl ComponentDeltas {
    // Check if  the current component was added
    pub fn was_added(&self) -> bool {
        self.state == ADDED_STATE
    }
    // Check if a component was mutated
    pub fn was_mutated(&self) -> bool {
        self.state == MUTATED_STATE
    }
    // Check if a component is pending for deletion
    pub fn is_pending_for_deletion(&self) -> bool {
        self.state == REMOVED_STATE
    }
}

// A linked component query. This can be iterated through in multiple threads for parallelism
// Even though the components are stored in an unsafe cell, this should never UB, since we never mutate a component while it is being written to
pub struct ComponentQuery {
    // The hashmap for ComponentBitmask -> Components
    components: Arc<RwLock<ComponentsHashMap>>,

    // Bitmask
    bitmask: u64,

    // The current bundle (entity) index
    bundle: usize,
}

// Trust trust
unsafe impl Send for ComponentQuery {}
unsafe impl Sync for ComponentQuery {}

impl ComponentQuery {
    // Create a new component query using a specific layout and a bundle index and archetype index
    pub(crate) unsafe fn new(
        set: &ArchetypeSet,
        bitmask: u64,
        bundle: usize,
        archetype: ArchetypeId,
    ) -> Self {
        // Temp
        let temp = set.get(archetype).unwrap();

        // Clone the components' arc
        let components = temp.components().clone();

        Self {
            components,
            bitmask,
            bundle,
        }
    }
    // Check if the component bits are valid first
    fn get_component_bits<T: Component>(&self) -> Result<u64, QueryError> {
        // Check if the bits are valid first
        let bits = T::bits().map_err(component_err::<T>)?;
        if self.bitmask & bits == 0 {
            return Err(unlinked_err::<T>())?;
        }
        Ok(bits)
    }
    // Get the component deltas
    pub fn deltas<T: Component>(&self) -> Result<ComponentDeltas, QueryError> {
        // Get the component bits for T
        let bits = self.get_component_bits::<T>()?;

        // Get the component deltas
        // Get the specific archetype component storages
        let lock = self.components.read();
        // Get the storage vector by downcasting
        let (_, states) = lock.get(&bits).unwrap();
        Ok(ComponentDeltas {
            state: states.get(self.bundle),
        })
    }
    // Get a component immutably
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        // Get the component bits for T
        let bits = self.get_component_bits::<T>()?;

        // Get the specific archetype component storages
        let lock = self.components.read();
        // Get the storage vector by downcasting
        let (storage, _) = lock.get(&bits).unwrap();
        let vec = storage
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<Vec<UnsafeCell<T>>>()
            .unwrap();
        // I want to apologize in advance for this
        Ok(unsafe { &*vec[self.bundle].get() })
    }
    // Get a component mutably
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        // Get the component bits for T
        let bits = self.get_component_bits::<T>()?;

        // Get the specific archetype component storages
        let lock = self.components.read();
        // Get the storage vector by downcasting
        let (storage, mutated) = lock.get(&bits).unwrap();
        let vec = storage
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<Vec<UnsafeCell<T>>>()
            .unwrap();

        // Don't overwrite removed or added
        mutated.set(self.bundle, MUTATED_STATE);

        // I want to apologize in advance for this
        Ok(unsafe { &mut *vec[self.bundle].get() })
    }
}