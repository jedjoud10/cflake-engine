use std::{
    any::{type_name, Any},
    cell::UnsafeCell,
    collections::{BTreeMap, HashMap},
    mem::size_of,
    sync::Arc,
};

use getset::{CopyGetters, Getters};
use parking_lot::RwLock;

use crate::{
    component::{registry, Component},
    entity::{Entity, EntityLinkings},
    prelude::{ComponentState, ComponentStatesBitfield, Mask},
};

use super::{
    ArchetypeError, ComponentStorage, ComponentStoragesHashMap, MaskHasher, UniqueComponentStoragesHashMap,
};

// The archetype set (BTreeMap)
pub type ArchetypeSet = BTreeMap<Mask, Archetype>;

// Combination of multiple component types
#[derive(Getters, CopyGetters)]
pub struct Archetype {
    // Component storage
    #[getset(get = "pub(crate)")]
    components: Arc<RwLock<ComponentStoragesHashMap>>,

    // Bundle Index -> Entity
    #[getset(get = "pub")]
    entities: Vec<Entity>,

    // Bundles that must be removed by the next iteration
    #[getset(get = "pub")]
    pending_for_removal: Vec<usize>,

    // Combined component masks
    #[getset(get_copy = "pub")]
    mask: Mask,
}

impl Archetype {
    // Create new a archetype based on it's individual component masks and it's combined mask
    pub(crate) fn new(masks: (&[Mask], Mask), uniques: &UniqueComponentStoragesHashMap) -> Self {
        // Use the uniquer component storages to make new empty storages
        let storages: ComponentStoragesHashMap = masks
            .0
            .iter()
            .map(|&id| {
                (
                    id,
                    (
                        uniques.get(&id).unwrap().new_empty_from_self(),
                        ComponentStatesBitfield::default(),
                    ),
                )
            })
            .collect();
        Self {
            components: Arc::new(RwLock::new(storages)),
            mask: masks.1,
            entities: Default::default(),
            pending_for_removal: Default::default(),
        }
    }

    // Insert an entity into the arhcetype using a ComponentLinker
    pub(crate) fn insert_with(
        &mut self,
        components: Vec<(Mask, Box<dyn Any>)>,
        linkings: &mut EntityLinkings,
        entity: Entity,
    ) {
        // Lock the component storages for writing
        let mut write = self.components.write();

        // Commons
        let len = self.entities.len() + 1;

        // Add the components using their specific storages
        for (mask, component) in components {
            let (storage, mutated) = write.get_mut(&mask).unwrap();

            
            // Update length
            mutated.set_len(len);
            // Set the new component state to Added
            mutated.set(len-1, ComponentState::Added);
            // Insert the component
            storage.push(component);
        }

        // Update the length
        self.entities.push(entity);
        linkings.bundle = self.entities.len() - 1;
        linkings.mask = self.mask;
    }

    // Start the deletion process for components. The component will actually get deleted next frame
    pub(crate) fn add_pending_for_removal(&mut self, bundle: usize) {
        // We don't need to have a write lock since the sparse bitfield is backed by atomics
        let read = self.components.read();

        // Just set the state of ComponentState::PendingForRemoval
        for (_, (_, mutated)) in read.iter() {
            // Set state
            mutated.set(bundle, ComponentState::PendingForRemoval);
        }

        // Pending for removal push
        self.pending_for_removal.push(bundle);
    }

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    // This will return the boxed components that were removed
    fn remove_boxed(&mut self, bundle: usize) -> Vec<(Mask, Box<dyn Any>)> {
        // The boxed components that will be returned
        let mut components: Vec<(Mask, Box<dyn Any>)> = Default::default();
        
        // Remove the components from the storages
        for (mask, (storage, _)) in self.components.write().iter_mut() {
            let boxed = storage.swap_remove_boxed_bundle(bundle);
            components.push((*mask, boxed));
        }

        // And then the locally stored entity ID
        self.entities.swap_remove(bundle);
        components
    }    

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    fn remove(&mut self, bundle: usize) {
        // Remove the components from the storages
        for (_, (storage, _)) in self.components.write().iter_mut() {
            storage.swap_remove_bundle(bundle);
        }

        // And then the locally stored entity ID
        self.entities.swap_remove(bundle);
    }

    // Remove all the components that are pending for removal
    fn remove_all_pending(&mut self) {
        // Steal
        let stolen = std::mem::take(&mut self.pending_for_removal);

        // And remove
        for bundle in stolen {
            self.remove(bundle);
        }
    }

    // Moves an entity from this archetype to another archetype
    pub(crate) fn move_entity(
        &mut self,
        entity: Entity,
        linkings: &mut EntityLinkings,
        other: &mut Self,
    ) {
        // First, remove the entity from Self directly
        let components = self.remove_boxed(linkings.bundle);

        // And insert into Other
        other.insert_with(components, linkings, entity);
    }

    // Prepare the arhcetype for execution. This will reset the component states, and remove the "pending for deletion" components
    pub fn prepare(&mut self) {
        // Remove "pending for deletion" components
        self.remove_all_pending();

        // Iterate through the bitfields and reset them
        let mut components = self.components.write();
        for (_, (_storage, states)) in components.iter_mut() {
            // Reset the states
            states.reset()
        }
    }
}
