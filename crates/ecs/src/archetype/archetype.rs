use super::UniqueComponentStoragesHashMap;
use crate::{
    entity::{Entity, EntityLinkings},
    ArchetypeStates, ComponentStorage, EntityState, Mask, MaskHasher,
};
use getset::{CopyGetters, Getters, MutGetters};
use std::{any::Any, collections::HashMap};
use tinyvec::ArrayVec;

// Combination of multiple component types
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Archetype {
    // Component vector
    #[getset(get = "pub(crate)", get_mut = "pub(crate)")]
    vectors: HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher>,

    // Bundle Index -> Entity
    #[getset(get = "pub")]
    entities: Vec<Entity>,

    // Stores the entity states and components states
    #[getset(get = "pub(crate)")]
    states: ArchetypeStates,

    // Bundles that must be removed by the next iteration
    #[getset(get = "pub")]
    pending_for_removal: Vec<usize>,

    // Combined component masks
    #[getset(get_copy = "pub")]
    mask: Mask,
}

impl Archetype {
    // Create new a archetype based on it's combined mask
    pub(crate) fn new(mask: Mask, uniques: &UniqueComponentStoragesHashMap) -> Self {
        // We must decompose the combined mask into the individual masks
        dbg!(mask);
        let masks = (0..(u64::BITS as usize))
            .into_iter()
            .filter_map(|i| {
                // Get the individual mask
                let individual = mask >> i;

                // Filter
                if individual & Mask::one() == Mask::one() {
                    Some((individual & Mask::one()) << i)
                } else {
                    None
                }
            })
            .collect::<ArrayVec<[Mask; 64]>>();

        // Use the unique component storages to make new empty vetors
        let vectors: HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher> = masks.iter().map(|mask| (*mask, uniques[mask].new_empty_from_self())).collect();
        Self {
            vectors,
            mask,
            states: Default::default(),
            entities: Default::default(),
            pending_for_removal: Default::default(),
        }
    }

    // Check if an entity is valid
    pub(crate) fn is_valid(&self, bundle: usize) -> bool {
        self.states.get_entity_state(bundle).unwrap() != EntityState::PendingForRemoval
    }

    // Insert an entity into the arhcetype using a ComponentLinker
    pub(crate) fn insert_with(&mut self, components: Vec<(Mask, Box<dyn Any>)>, linkings: &mut EntityLinkings, entity: Entity) {
        // Push first
        self.entities.push(entity);
        self.states.push();        
        linkings.bundle = self.entities.len() - 1;
        linkings.mask = self.mask;
        
        // Add the components using their specific storages
        for (mask, component) in components {
            let vec = self.vectors.get_mut(&mask).unwrap();
            
            // Insert the component
            vec.push(component);
            self.states.set_component_state(linkings.bundle, mask, true);
        }
    }

    // Start the deletion process for components. The component will actually get deleted next frame
    pub(crate) fn add_pending_for_removal(&mut self, bundle: usize) {
        // Pending for removal push
        self.pending_for_removal.push(bundle);

        // Set the entity state
        self.states.set_entity_state(bundle, EntityState::PendingForRemoval);
    }

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    // This will return the boxed components that were removed, but only the ones that validate the given mask
    fn remove_boxed_filtered(&mut self, bundle: usize, filter_mask: Mask) -> Vec<(Mask, Box<dyn Any>)> {
        // The boxed components that will be returned
        let mut components: Vec<(Mask, Box<dyn Any>)> = Default::default();

        // Remove the components from the storages
        for (&mask, vec) in self.vectors.iter_mut() {
            // Filter the components that validate the mask
            if mask & filter_mask == mask {
                let boxed = vec.swap_remove_boxed_bundle(bundle);
                components.push((mask, boxed));
            }
        }

        // And then the locally stored entity ID
        self.entities.swap_remove(bundle);
        components
    }

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    fn remove(&mut self, bundle: usize) {
        // Remove the components from the storages
        for (_, vec) in self.vectors.iter_mut() {
            vec.swap_remove_bundle(bundle);
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
    // We will also be able to add some extra components if needed
    pub(crate) fn move_entity(&mut self, entity: Entity, linkings: &mut EntityLinkings, extra: Vec<(Mask, Box<dyn Any>)>, other: &mut Self) {
        // First, remove the entity from Self directly
        let mut components = self.remove_boxed_filtered(linkings.bundle, other.mask);

        // Combine the removed components with the extra components
        components.extend(extra);

        // And insert into Other
        other.insert_with(components, linkings, entity);
    }

    // Prepare the arhcetype for execution. This will reset the component states, and remove the "pending for deletion" components
    pub fn prepare(&mut self, count: u64) {
        // Don't do anything for the first frame of execution
        if count == 0 { return; }

        // Remove "pending for deletion" components
        self.remove_all_pending();

        // Reset the component and entity states
        self.vectors.iter().for_each(|(m, _)| self.states.reset_component_states(*m));
        self.states.reset_entity_states();
    }
}
