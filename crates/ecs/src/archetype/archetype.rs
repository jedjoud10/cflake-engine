use std::{any::{type_name, Any}, cell::UnsafeCell, sync::Arc, collections::HashMap};

use getset::Getters;
use parking_lot::RwLock;

use crate::{
    archetype::duplicate_err,
    component::{Component, ComponentLayout, SparseComponentStates, ADDED_STATE, registry},
    entity::{Entity, EntityLinkings},
};

use super::{component_err, invalid_er, ArchetypeError, ArchetypeId, ComponentsHashMap, MaybeNoneStorage, ArchetypeSet, NoHash};

// Combination of multiple component types
#[derive(Getters)]
pub struct Archetype {
    // Component storage
    #[getset(get = "pub")]
    components: Arc<RwLock<ComponentsHashMap>>,

    // How many components stored per storage
    length: usize,

    // Bundles -> Entity ID
    entities: Vec<Entity>,

    // Component bits
    #[getset(get = "pub")]
    bits: u64,
}

impl Archetype {
    // Get the entities that are stored into this archetype
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
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
        let storages: ComponentsHashMap = layout.bits.iter().map(|&id| (id, (None, SparseComponentStates::default()))).collect();
        Self {
            components: Arc::new(RwLock::new(storages)),
            length: 0,
            bits: layout.mask,
            entities: Vec::new(),
        }
    }

    // Insert an entity into the arhcetype using a ComponentLinker
    pub(crate) fn insert_with(&mut self, components: HashMap<u64, Box<dyn Component>, NoHash>, linkings: &mut EntityLinkings, entity: Entity) {
        // Add the components using their specific bits
        for (bitmask, component) in components {

        }



        // Update the length
        self.length += 1;
        linkings.bundle = self.length - 1;
        linkings.archetype = ArchetypeId(self.bits);
        Ok(())
    }

    // Insert a component into the archetype
    // This must be called whenever we are inserting components for an entity
    fn insert<T: Component>(&mut self, component: Box<dyn Any>) -> Result<(), ArchetypeError> {
        // We must register the components separately
        let bits = registry::bits::<T>().map_err(component_err::<T>)?;

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
        let storage = storage.as_any_mut().downcast_mut::<Vec<UnsafeCell<T>>>().unwrap();

        // Insert the component
        let component = *component.downcast::<T>().unwrap();
        Ok(storage.push(UnsafeCell::new(component)))
    }

    // Reset the archetype for the next frame. This will cear the mutated components' flags
    pub fn prepare(&mut self) {
        // Iterate through the bitfields and reset them
        let mut components = self.components.write();
        for (_, (_, mutated)) in components.iter_mut() {
            mutated.reset();
        }
    }
}

// Something that we can use to link components to an entity
pub struct ComponentLinker<'a> {
    // Archetype set
    set: &'a mut ArchetypeSet,

    // The stored components
    components: HashMap<u64, Box<dyn Any>, NoHash>,

    // Bits of the components that were successfully added
    bits: u64,
} 

impl<'a> ComponentLinker<'a> {
    // Add a single component into this local inserter
    pub fn insert<T: Component>(&mut self, component: T) -> Result<(), ArchetypeError> {
        // Only insert the component if we never had it before
        let component_bits = registry::bits::<T>().map_err(component_err::<T>)?;
        let new = self.bits | component_bits;

        // Check for link duplication
        if self.bits == new { return Err(duplicate_err::<T>()); }
        self.bits = new;

        // Temporarily store the components
        self.components.insert(component_bits, Box::new(component));
        Ok(())
    }
}
