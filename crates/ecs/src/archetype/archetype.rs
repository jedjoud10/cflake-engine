use crate::{
    entity::{Entity, EntityLinkings},
    registry, Component, ComponentError, ComponentStateSet, Mask, MaskMap, StorageVec, UniqueStoragesSet, ArchetypeSet,
};
use getset::{CopyGetters, Getters, MutGetters};
use std::{any::Any, ffi::c_void, ptr::NonNull, rc::Rc};
use tinyvec::ArrayVec;

type ComponentColumns = MaskMap<(Box<dyn StorageVec>, NonNull<c_void>)>;

// Combination of multiple component types
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Archetype {
    // Main
    pub(crate) mask: Mask,

    // Components
    pub(crate) vectors: ComponentColumns,
    pub(crate) states: Rc<ComponentStateSet>,

    // Entities
    pub(crate) length: usize,

    // Others
    pub(crate) cache_index: Option<usize>,
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
                (*mask, uniques[mask].clone_unique_storage())
            })
            .collect();
        Self {
            vectors,
            mask,
            states: Default::default(),
            length: 0,
            cache_index: None,
        }
    }

    // Add an entity into the archetype and update it's linkings
    pub(crate) fn push(&mut self, linkings: &mut EntityLinkings, entity: Entity) {
        self.length += 1;
        self.states.push();
        linkings.bundle = self.length - 1;
        linkings.mask = self.mask;
    }

    // Insert a component direclty into the archetype storage
    // It is up to the calling function to make sure that all the component storage lengths are synced later
    pub(crate) fn insert_component<T: Component>(&mut self, component: T) -> Result<(), ComponentError> {
        let mask = registry::mask::<T>()?;

        // Cast to the actual vector, then push
        let (boxed, ptr) = self.vectors.get_mut(&mask).unwrap();
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        self.states.set(vec.len(), mask);

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
            self.states.set(linkings.bundle, mask);
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

    // Directly removes a bundle from the archetype
    pub(crate) fn remove(&mut self, bundle: usize) {
        // Remove the components from the storages
        for (_, (vec, _)) in self.vectors.iter_mut() {
            vec.swap_remove(bundle);
        }

        // And then the locally stored entity ID
        self.entities.swap_remove(bundle);
    }

    // Prepare the arhcetype for execution. This will reset the component states, and remove the "pending for deletion" components
    pub(crate) fn prepare(&mut self, count: u64) {
        // Don't do anything for the first frame of execution
        if count == 0 {
            return;
        }

        // Reset the deltas/states that were set during the execution frame
        self.states.reset_to(false);
    }
}


// Move an entity from an archetype to another archetype, whilst adding extra components to the entity
// If the old and new masks are not disjoint, this will UB
pub(crate) unsafe fn move_entity(archetypes: &mut ArchetypeSet, old: Mask, new: Mask, entity: Entity, linkings: &mut EntityLinkings, extra: Vec<(Mask, Box<dyn Any>)>) {
    // A bit of unsafe code but this should technically still be safe
    let ptr1: *mut Archetype = archetypes.get_mut(&old).unwrap();
    let ptr2: *mut Archetype = archetypes.get_mut(&new).unwrap();
    let (old, new) = (&mut *ptr1, &mut *ptr2);

    // The boxed components that will be added into the new archetype
    let mut components: Vec<(Mask, Box<dyn Any>)> = Vec::with_capacity(new.mask.count_ones() as usize);

    // Remove the components from the storages
    for (&mask, (vec, _)) in old.vectors.iter_mut() {
        // Filter the components that validate the mask
        if mask & new.mask == mask {
            let boxed = vec.
            (linkings.bundle);
            components.push((mask, boxed));
        }
    }

    // And then the locally stored entity ID
    self.entities.swap_remove(bundle);

    // Combine the removed components with the extra components
    components.extend(extra);
    
    // And insert into Other
    other.insert_boxed(components, linkings, entity);
}