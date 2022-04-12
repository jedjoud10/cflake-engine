use crate::{
    entity::{Entity, EntityLinkings},
    registry, Component, ComponentError, ComponentStateSet, Mask, MaskMap, StorageVec, UniqueStoragesSet,
};
use getset::{CopyGetters, Getters, MutGetters};
use std::{any::Any, collections::HashMap, ffi::c_void, rc::Rc};
use tinyvec::ArrayVec;

type ComponentColumns = MaskMap<(Box<dyn StorageVec>, *mut c_void)>;

// Combination of multiple component types
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Archetype {
    // Component vector
    pub(crate) vectors: ComponentColumns,

    // Bundle Index -> Entity
    pub(crate) entities: Vec<Entity>,

    // Component mutation states
    pub(crate) states: Rc<ComponentStateSet>,

    // Bundles that must be removed by the next iteration
    pub(crate) pending_for_removal: Vec<usize>,

    // Index of this archetype inside the query cache
    pub(crate) cache_index: usize,

    // Check if we need to update the cache chunk that is in relation with this archetype
    pub(crate) cache_pending_update: bool,

    // Combined component masks
    pub(crate) mask: Mask,
}

impl Archetype {
    // Create new a archetype based on it's combined mask
    pub(crate) fn new(mask: Mask, uniques: &UniqueStoragesSet) -> Self {
        // We must decompose the combined mask into the individual masks
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
        let vectors: ComponentColumns = masks
            .iter()
            .map(|mask| {
                let boxed = uniques[mask].new_empty_from_self();
                let ptr = boxed.get_null_mut_typeless_ptr();
                (*mask, (boxed, ptr))
            })
            .collect();
        Self {
            vectors,
            mask,
            states: Default::default(),
            entities: Default::default(),
            cache_index: 0,
            cache_pending_update: true,
            pending_for_removal: Default::default(),
        }
    }

    // Add an entity by itself and updates it's linkings
    pub(crate) fn push_entity(&mut self, linkings: &mut EntityLinkings, entity: Entity) {
        let old = self.entities.capacity();
        self.entities.push(entity);
        self.states.push();
        linkings.bundle = self.entities.len() - 1;
        linkings.mask = self.mask;

        // Check if we've reallocated the vectors
        // Since the lengths of the component vectors and entity vector are synced, when one reallocates, the other will surely reallocate
        self.cache_pending_update |= self.entities.capacity() > old;
    }

    // Insert a component direclty into the archetype storage
    // It is up to the calling function to make sure that all the component storage lengths are synced later
    pub(crate) fn insert_component<T: Component>(&mut self, component: T) -> Result<(), ComponentError> {
        let mask = registry::mask::<T>()?;

        // Cast to the actual vector, then push
        let (boxed, ptr) = self.vectors.get_mut(&mask).unwrap();
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        self.states.set(true, vec.len(), mask);

        // Push and update the pointer
        vec.push(component);
        *ptr = vec.as_mut_typeless_ptr();
        Ok(())
    }

    // Insert an entity into the arhcetype using a ComponentLinker
    pub(crate) fn insert_boxed(&mut self, components: Vec<(Mask, Box<dyn Any>)>, linkings: &mut EntityLinkings, entity: Entity) {
        self.push_entity(linkings, entity);
        // Add the components using their specific storages
        for (mask, component) in components {
            let (vec, ptr) = self.vectors.get_mut(&mask).unwrap();

            // Insert the component (and update the pointer if it changed)
            vec.push(component);
            *ptr = vec.as_mut_typeless_ptr();
            //self.states.set_component_state(linkings.bundle, mask, true);
        }
    }

    // Reserve enough space to fit "n" number of new entities into this archetype
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.states.reserve(additional);

        // Reallocate if needed
        for (_, (vec, ptr)) in self.vectors.iter_mut() {
            vec.reserve(additional);
            *ptr = vec.as_mut_typeless_ptr();
        }
    }

    // Start the deletion process for components. The component will actually get deleted next frame
    pub(crate) fn add_pending_for_removal(&mut self, bundle: usize) {
        // Pending for removal push
        self.pending_for_removal.push(bundle);
    }

    // Directly removes a bundle from the archetype (PS: This mutably locks "components")
    // This will return the boxed components that were removed, but only the ones that validate the given mask
    fn remove_boxed_filtered(&mut self, bundle: usize, filter_mask: Mask) -> Vec<(Mask, Box<dyn Any>)> {
        // The boxed components that will be returned
        let mut components: Vec<(Mask, Box<dyn Any>)> = Default::default();

        // Remove the components from the storages
        for (&mask, (vec, _)) in self.vectors.iter_mut() {
            // Filter the components that validate the mask
            if mask & filter_mask == mask {
                let boxed = vec.swap_remove_boxed(bundle);
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
        for (_, (vec, _)) in self.vectors.iter_mut() {
            vec.swap_remove(bundle);
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
        other.insert_boxed(components, linkings, entity);
    }

    // Prepare the arhcetype for execution. This will reset the component states, and remove the "pending for deletion" components
    pub(crate) fn prepare(&mut self, count: u64) {
        // Don't do anything for the first frame of execution
        if count == 0 {
            return;
        }

        // Remove "pending for deletion" components
        self.remove_all_pending();

        // Reset the component states
        self.states.reset();
    }
}
