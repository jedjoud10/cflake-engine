use std::{any::type_name, cell::UnsafeCell, sync::Arc};

use getset::Getters;
use parking_lot::RwLock;

use crate::{
    archetype::duplicate_err,
    component::{Component, ComponentLayout, SparseComponentStates, ADDED_STATE},
};

use super::{
    component_err, invalid_er, ArchetypeError, ArchetypeId, ComponentsHashMap, MaybeNoneStorage,
};

// Combination of multiple component types
#[derive(Getters)]
pub struct Archetype {
    // Component storage
    #[getset(get = "pub")]
    components: Arc<RwLock<ComponentsHashMap>>,

    // How many components stored per storage
    length: usize,

    // Component bits
    #[getset(get = "pub")]
    bits: u64,
}

impl Archetype {
    // Length and is empty
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Archetype {
    // Create new a archetype based on it's layout
    pub(super) fn new(layout: ComponentLayout) -> Self {
        // Allocate all the new component storages
        let storages: ComponentsHashMap = layout
            .bits
            .iter()
            .map(|&id| (id, (None, SparseComponentStates::default())))
            .collect();
        Self {
            components: Arc::new(RwLock::new(storages)),
            length: 0,
            bits: layout.mask,
        }
    }

    // Insert multiple components in one go, and return the specific bundle index
    pub(crate) fn insert_with(
        &mut self,
        callback: impl FnOnce(&mut ArchetypeStorage),
    ) -> Result<usize, ArchetypeError> {
        // Run the callback first
        let mut storage = ArchetypeStorage {
            archetype: self,
            bits: 0,
        };
        callback(&mut storage);

        // Check if we added ALL the components that satisfy the layout
        if storage.bits != self.bits {
            return Err(ArchetypeError::IncompleteLinks);
        }

        // Update the length
        self.length += 1;
        Ok(self.length - 1)
    }

    // Insert a component into the archetype
    // This must be called whenever we are inserting components for an entity
    fn insert<T: Component>(&mut self, component: T) -> Result<(), ArchetypeError> {
        // We must register the components separately
        let bits = T::bits().map_err(component_err::<T>)?;

        // Check if the component is even valid to be stored inside the archetype
        if self.bits & bits == 0 {
            return Err(invalid_er::<T>());
        }

        // Lock
        let mut lock = self.components.write();

        // Get the proper storage, and push the element
        let (storage, mutated) = lock.get_mut(&bits).unwrap();

        // Set the component's flags
        mutated.extend_by_one();
        mutated.set(self.len(), ADDED_STATE);

        // Insert the storage if it does not exist yet
        let storage = storage.get_or_insert(Box::new(Vec::<UnsafeCell<T>>::new()));

        // Cast to the approriate type now
        let storage = storage
            .as_any_mut()
            .downcast_mut::<Vec<UnsafeCell<T>>>()
            .unwrap();

        // Insert the component
        Ok(storage.push(UnsafeCell::new(component)))
    }

    // Reset the archetype for the next frame. This will cear the mutated components' bitfield
    pub fn prepare(&mut self) {
        // Iterate through the bitfields and reset them
        let mut components = self.components.write();
        for (_, (_, mutated)) in components.iter_mut() {
            mutated.reset();
        }
    }
}

// Helper struct for inserting components
pub struct ArchetypeStorage<'a> {
    // Our archetype
    archetype: &'a mut Archetype,

    // Bits of the components that were successfully added
    bits: u64,
}
impl<'a> ArchetypeStorage<'a> {
    // Insert a single component into the archetype
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), ArchetypeError> {
        // Only insert the component if we never had it before
        let new = self.bits | T::bits().map_err(component_err::<T>)?;

        // Check for link duplication
        if self.bits == new {
            return Err(duplicate_err::<T>());
        }
        self.bits = new;
        self.archetype.insert(component)
    }
}
