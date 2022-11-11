use crate::{
    entity::{Entity, EntityLinkings},
    mask, ArchetypeSet, Bundle, Component, ComponentTable, EntitySet, Mask, MaskHashMap,
    QueryLayoutRef,
};
use std::{cell::RefCell, rc::Rc};

// A single chunk that will be contained within the archetype component column
#[derive(Default, Clone, Copy)]
pub struct StateColumnChunk {
    added: u64,
    removed: u64,
    modified: u64,
}

// Returned from the Vec<StateColumnChunk>
#[derive(Default, Clone, Copy)]
pub struct StateFlags {
    added: bool,
    removed: bool,
    modified: bool,
}

// An archetype is a special structure that contains multiple entities of the same layout
// Archetypes are used in archetypal ECSs to improve iteration and insertion/removal performance
pub struct Archetype {
    mask: Mask,
    components: MaskHashMap<Box<dyn ComponentTable>>,
    states: MaskHashMap<Vec<StateColumnChunk>>,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create a new archetype from a owned bundle accessor
    // This assumes that B is a valid bundle
    pub(crate) fn from_table_accessor<B: Bundle>() -> Self {
        Self {
            mask: B::reduce(|a, b| a | b),
            components: B::default_tables(),
            states: Default::default(),
            entities: Vec::new(),
        }
    }

    // Create the unit archetype that contains no tables and has a zeroed mask
    pub(crate) fn empty() -> Self {
        Self {
            mask: Mask::zero(),
            components: Default::default(),
            states: Default::default(),
            entities: Default::default(),
        }
    }

    // Add multiple entities into the archetype with their corresponding owned components
    // The layout mask for "B" must be equal to the layout mask that this archetype contains
    pub(crate) fn extend_from_slice<B: Bundle>(
        &mut self,
        entities: &mut EntitySet,
        components: Vec<B>,
    ) -> &[Entity] {
        debug_assert_eq!(self.mask(), B::reduce(|a, b| a | b));
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Reserve and calculate difference
        self.reserve(entities.len());
        let old_len = self.entities.len();
        let new_len = self.entities.len();
        let additional = components.len();

        // Add the entities internally and externally
        for _ in 0..additional {
            let linkings = EntityLinkings {
                mask: self.mask,
                index: self.len(),
            };
            let entity = entities.insert(linkings);
            self.entities.push(entity);
        }

        // Add the state bits if needed
        for (_, column)  in self.states.iter_mut() {
            // Make sure the states have enough chunks to deal with
            let iter = std::iter::repeat(StateColumnChunk::default());
            let iter = iter.take(additional / u64::BITS as usize);
            column.extend(iter);

            // Update the chunk bits
            for (i, chunk) in column.iter_mut().enumerate() {
                let start = i * u64::BITS as usize;
                let local_start = usize::saturating_sub(old_len, start).min(u64::BITS as usize);
                let local_end = usize::saturating_sub(new_len, start).min(u64::BITS as usize);

                // Bit magic that will enable all the bits between local_start and local_end;
                let range = ((1u64 << (local_start + 1)) - 1u64) ^ ((1u64 << local_end) - 1u64);
                chunk.added |= range;
                chunk.modified |= range;
            }
        }
        
        // Add the storage bundles to their respective tables
        let mut storages = B::prepare(self).unwrap();
        for set in components {
            B::push(&mut storages, set);
        }
        drop(storages);

        // Return the newly added entity IDs
        &self.entities[old_len..]
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.states.reserve(additional);

        for (_, column) in &mut self.components {
            column.reserve(additional);
        }

        for (_, column) in &mut self.states {
            let ceiled = (additional as f32 / u64::BITS as f32).ceil();
            column.reserve(ceiled as usize);
        }
    }

    // Get the number of entities that reference this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    // Get the entity slice immutably
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // Get the unique archetype mask
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Try to get an immutable reference to the table for a specific component
    pub fn table<T: Component>(&self) -> Option<&Vec<T>> {
        let boxed = &self.columns.get(&mask::<T>())?.data;
        Some(boxed.as_any().downcast_ref().unwrap())
    }

    // Try to get a mutable reference to the table for a specific component
    pub fn table_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        let boxed = &mut self.columns.get_mut(&mask::<T>())?.data;
        Some(boxed.as_any_mut().downcast_mut().unwrap())
    }

    // Remove an entity that is stored within this archetype using it's index
    // This will return the entity's old linkings if successful
    pub(crate) fn remove(
        &mut self,
        entities: &mut EntitySet,
        entity: Entity,
    ) -> Option<EntityLinkings> {
        // Try to get the linkings and index
        let linkings = entities.remove(entity)?;
        let index = linkings.index();

        // Remove the components from the tables
        for (_, column) in self.columns.iter_mut() {
            column.swap_remove(index)
        }

        // Remove the entity and get the entity that was swapped with it
        self.entities.swap_remove(index);
        let entity = self.entities.get(index).cloned();

        // Swap might've failed if we swapped with the last element in the vector
        if let Some(entity) = entity {
            let swapped = entities.get_mut(entity).unwrap();
            swapped.index = index;
        }

        Some(linkings)
    }
}

// This will get two different archetypes using their masks
// This assumes that the archetypes exist already in the set, and that we are using different masks
fn split(set: &mut ArchetypeSet, mask1: Mask, mask2: Mask) -> (&mut Archetype, &mut Archetype) {
    assert_ne!(mask1, mask2);
    let a1 = set.get_mut(&mask1).unwrap() as *mut Archetype;
    let a2 = set.get_mut(&mask2).unwrap() as *mut Archetype;
    unsafe {
        let a1 = &mut *a1;
        let a2 = &mut *a2;
        (a1, a2)
    }
}

// Add some new components onto an entity, forcing it to switch archetypes
pub(crate) fn add_bundle_unchecked<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
    bundle: B,
) -> Option<()> {
    assert!(
        B::is_valid(),
        "Bundle is not valid, check the bundle for component collisions"
    );

    // Get the old and new masks
    let old = entities[entity].mask;
    let new = entities[entity].mask | B::reduce(|a, b| a | b);

    // Nothing changed, don't execute
    if new == old {
        return Some(());
    }

    // Create the new target archetype if needed
    if archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let columns = current
            .columns
            .iter()
            .map(|(mask, column)| (*mask, ArchetypeColumn {
                data: column.data.clone_default(),
                states: Vec::default(),
            }));
        let archetype = Archetype {
            mask: new,
            columns: MaskHashMap::from_iter(columns),
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.remove(entity)?;
    let index = linkings.index();

    // Move the components from one archetype to the other
    for (mask, input) in current.columns.iter_mut() {
        let output = target.columns.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
        input.update_state_entry(index, |current| {
            current.added = true;
            current.modified = true;
        });
    }

    // Add the extra components as well
    let mut storages = B::prepare(target)?;
    B::push(&mut storages, bundle);
    drop(storages);

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target.entities.push(entity);
    linkings.index = target.len() - 1;
    linkings.mask = target.mask;

    Some(())
}

// Remove some old components from an entity, forcing it to switch archetypes
// This assumes that the OwnedBundle type is valid for this use case
pub(crate) fn remove_bundle_unchecked<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
) -> Option<B> {
    assert!(B::is_valid(), "Bundle is not valid");

    // Get the old and new masks
    let old = entities[entity].mask;
    let combined = B::reduce(|a, b| a | b);
    let new = entities[entity].mask & !combined;

    // Create the new target archetype if needed
    if archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let columns = current
            .columns
            .iter()
            .map(|(mask, column)| (*mask, ArchetypeColumn {
                data: column.data.clone_default(),
                states: Vec::default(),
            }));
        let filtered = columns.filter(|(mask, _)| Mask::contains(&new, *mask));
        let archetype = Archetype {
            mask: new,
            columns: MaskHashMap::from_iter(columns),
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.remove(entity)?;
    let index = linkings.index();

    // Move the components from one archetype to the other (swapped)
    for (mask, output) in target.columns.iter_mut() {
        let input = current.columns.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
        output.update_state_entry(index, |current| {
            current.removed = true;
        });
    }

    // Create the return bundle
    let bundle = B::try_swap_remove(&mut current.columns, index);

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target.entities.push(entity);
    linkings.index = target.len() - 1;
    linkings.mask = target.mask;

    bundle
}
