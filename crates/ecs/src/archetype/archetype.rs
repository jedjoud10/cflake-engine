use crate::{
    entity::{Entity, EntityLinkings},
    registry, ArchetypeSet, Component, ComponentError, ComponentStateSet, EntitySet, Mask, MaskMap, StorageVec, UniqueStoragesSet,
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
    pub(crate) index: Option<usize>,
}

impl Archetype {
    // Create new a archetype based on it's main mask
    pub(crate) fn new(mask: Mask, uniques: &UniqueStoragesSet) -> Self {
        // We must decompose the combined mask into the individual masks and create the storages from that
        let vectors = (0..(registry::count() as usize))
            .into_iter()
            .filter_map(|i| {
                // Make sure the bit is valid
                if (mask >> i) & Mask::one() != Mask::one() { return None; }

                // Create the archetype storage
                let mask = Mask::one() << i;
                Some((mask, uniques[&mask].clone_unique_storage()))
            })
            .collect::<ComponentColumns>();

        Self {
            vectors,
            mask,
            states: Default::default(),
            length: 0,
            index: None,
        }
    }

    // Add an entity into the archetype and update it's linkings
    pub(crate) fn push(&mut self, linkings: &mut EntityLinkings, components: Vec<(Mask, Box<dyn Any>)>) {
        // Add the entity and update it's linkings
        self.length += 1;
        self.states.push();
        linkings.bundle = self.length - 1;
        linkings.mask = self.mask;

        // Add the components using their specific storages
        for (mask, component) in components {
            let (vec, ptr) = self.vectors.get_mut(&mask).unwrap();

            // Insert the component (and update the pointer if it changed)
            vec.push(component);
            *ptr = vec.as_mut_typeless_ptr();
            self.states.set(linkings.bundle, mask);
        }
    }

    // Update all the underlying storages with a closure ran over them
    fn update_storages(&mut self, mut function: impl FnMut(&mut Box<dyn StorageVec>)) {
        for (_, (vec, ptr)) in self.vectors.iter_mut() {
            function(vec);

            // Might've reallocated, we don't know really
            *ptr = vec.as_mut_typeless_ptr();
        }
    }

    // Reserve enough space to fit "n" number of new entities into this archetype
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.states.reserve(additional);
        self.update_storages(|vec| vec.reserve(additional));
        
    }

    // Remove an entity from the archetype. This will update the linkings of another entity in the set
    pub(crate) fn remove(&mut self, bundle: usize, set: &mut EntitySet) {
        // Remove the components from the storages
        for (_, (vec, _)) in self.vectors.iter_mut() {
            vec.swap_remove(bundle);
        }

        // Handle the new swap index shit fuckery AAAAA
        panic!()
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
pub(crate) unsafe fn move_entity(archetypes: &mut ArchetypeSet, old: Mask, new: Mask, linkings: &mut EntityLinkings, extra: Vec<(Mask, Box<dyn Any>)>) {
    // A bit of unsafe code but this should technically still be safe
    let ptr1: *mut Archetype = archetypes.get_mut(&old).unwrap();
    let ptr2: *mut Archetype = archetypes.get_mut(&new).unwrap();
    let (old, new) = (&mut *ptr1, &mut *ptr2);

    // The boxed components that will be added into the new archetype
    let mut components: Vec<(Mask, Box<dyn Any>)> = Vec::with_capacity(new.mask.count_ones() as usize + extra.len());

    // Remove the components from the storages
    for (&mask, (vec, _)) in old.vectors.iter_mut() {
        // Filter the components that validate the mask
        if mask & new.mask == mask {
            // Remove the component, and box it
            components.push((mask, vec.swap_remove_boxed(linkings.bundle)));
        } else {
            // Remove it normally
            vec.swap_remove(linkings.bundle);
        }
    }

    // Remove the entity (this might fail in the case of the default empty archetype)
    new.length = new.length.saturating_sub(1);

    // Combine the removed components with the extra components
    components.extend(extra);

    // And insert into the new archetype
    new.push(linkings, components);
}
