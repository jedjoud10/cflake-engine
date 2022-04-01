use std::{any::Any, collections::HashMap, sync::Arc};
use getset::{CopyGetters, Getters};
use tinyvec::ArrayVec;
use super::{UniqueComponentStoragesHashMap, states::EntityStatesBitfield};
use crate::{
    entity::{Entity, EntityLinkings}, Mask, archetype::states::{ComponentMutationsBitfield, EntityState}, ComponentStorage, MaskHasher, ArchetypeStates,
};

// Combination of multiple component types
#[derive(Getters, CopyGetters)]
pub struct Archetype {
    // Component vector
    #[getset(get = "pub(crate)")]
    vector: HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher>,

    // Component and entity states
    states: ArchetypeStates,

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
    // Create new a archetype based on it's combined mask
    pub(crate) fn new(mask: Mask, uniques: &UniqueComponentStoragesHashMap) -> Self {
        // We must decompose the combined mask into the individual masks
        dbg!(mask);
        let masks = (0..(u64::BITS as usize)).into_iter().filter_map(|i| {
            // Get the individual mask
            let individual = mask >> i;

            // Filter
            if individual & Mask::one() == Mask::one() {
                Some((individual & Mask::one()) << i)
            } else {
                None
            }
        }).collect::<ArrayVec<[Mask; 64]>>();

        // Use the unique component storages to make new empty vetors
        let vector: HashMap<Mask, Box<dyn ComponentStorage>, MaskHasher> = masks
            .iter()
            .map(|mask| {
                (*mask, uniques.get(mask).unwrap().new_empty_from_self())
            })
            .collect();
        Self {
            vector,
            mask,
            entities: Default::default(),
            states: ArchetypeStates::new(masks.into_iter()),
            pending_for_removal: Default::default(),
        }
    }

    // Insert an entity into the arhcetype using a ComponentLinker
    pub(crate) fn insert_with(&mut self, components: Vec<(Mask, Box<dyn Any>)>, linkings: &mut EntityLinkings, entity: Entity) {
        // Commons
        let len = self.entities.len() + 1;
        dbg!(self.mask);
        // Add the components using their specific storages
        for (mask, component) in components {
            dbg!(mask);
            let (vec, mutated) = self.components.get_mut(&mask).unwrap();

            // Insert the component
            mutated.set_len(len);
            mutated.set_mutated_state(len - 1);
            vec.push(component);
        }

        // Set the entity state
        self.states.set_len(len);
        self.states.set(len - 1, EntityState::Added);

        // Update the length
        self.entities.push(entity);
        linkings.bundle = self.entities.len() - 1;
        linkings.mask = self.mask;        
    }

    // Start the deletion process for components. The component will actually get deleted next frame
    pub(crate) fn add_pending_for_removal(&mut self, bundle: usize) {
        // Pending for removal push
        self.pending_for_removal.push(bundle);

        // Set the entity state
        self.states.set(bundle, EntityState::PendingForRemoval)
    }

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    // This will return the boxed components that were removed, but only the ones that validate the given mask
    fn remove_boxed_filtered(&mut self, bundle: usize, filter_mask: Mask) -> Vec<(Mask, Box<dyn Any>)> {
        // The boxed components that will be returned
        let mut components: Vec<(Mask, Box<dyn Any>)> = Default::default();

        // Remove the components from the storages
        for (&mask, (storage, _)) in self.components.iter_mut() {
            // Filter the components that validate the mask
            if mask & filter_mask == mask {
                let boxed = storage.swap_remove_boxed_bundle(bundle);
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
        for (_, (storage, _)) in self.components.iter_mut() {
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
    // We will also be able to add some extra components if needed
    pub(crate) fn move_entity(&mut self, entity: Entity, linkings: &mut EntityLinkings, extra: Vec<(Mask, Box<dyn Any>)>, other: &mut Self) {
        // First, remove the entity from Self directly
        let mut components = self.remove_boxed_filtered(linkings.bundle, other.mask);

        // Combine the removed components with the extra components
        components.extend(extra);

        println!("{}", components.len());

        // And insert into Other
        other.insert_with(components, linkings, entity);
    }

    // Prepare the arhcetype for execution. This will reset the component states, and remove the "pending for deletion" components
    pub fn prepare(&mut self) {
        // Remove "pending for deletion" components
        self.remove_all_pending();

        // Iterate through the bitfields and reset them
        for (_, (_storage, states)) in self.components.iter_mut() {
            // Reset the states
            states.reset()
        }

        // Also reset the entity states
        self.states.reset();
    }
}
