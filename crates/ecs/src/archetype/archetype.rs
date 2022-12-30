use std::any::TypeId;

use crate::{
    entity::{Entity, EntityLinkings},
    mask, ArchetypeSet, Bundle, Component,
    EntitySet, Mask, MaskHashMap, QueryLayoutRef, StateColumn,
    StateFlags, Column, UntypedColumn,
};

// The table that will be stored internally
pub type Table = MaskHashMap<Box<dyn UntypedColumn>>;

// An archetype is a special structure that contains multiple entities of the same layout
// Archetypes are used in archetypal ECSs to improve iteration and insertion/removal performance
pub struct Archetype {
    mask: Mask,

    // Table that contains the columns that themselves contain the components and the states
    table: Table,

    // Entities that have the same mask as the archetype's mask
    entities: Vec<Entity>,
}

impl Archetype {
    // Create a new archetype from a owned bundle accessor
    // This assumes that B is a valid bundle
    pub(crate) fn from_bundle<B: Bundle>() -> Self {
        let mask = B::reduce(|a, b| a | b);

        println!("Creating archetype from bundle of mask {:?}", mask);

        Self {
            mask,
            table: B::default_tables(),
            entities: Vec::new(),
        }
    }

    // Create the unit archetype that contains no columns and has a zeroed mask
    pub(crate) fn empty() -> Self {
        Self {
            mask: Mask::zero(),
            table: Default::default(),
            entities: Default::default(),
        }
    }

    // Add multiple entities into the archetype with their corresponding owned components
    // The layout mask for "B" must be equal to the layout mask that this archetype contains
    pub(crate) fn extend_from_slice<B: Bundle>(
        &mut self,
        entities: &mut EntitySet,
        components: impl IntoIterator<Item = B>,
    ) -> &[Entity] {
        assert_eq!(self.mask(), B::reduce(|a, b| a | b));
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Reserve and calculate difference
        let iter = components.into_iter();
        self.reserve(iter.size_hint().0);
        let old_len = self.entities.len();

        // Add the components first (so we know how many entities we need to add)
        let mut storages = B::prepare(self).unwrap();
        let mut additional = 0;
        for set in iter {
            set.push(&mut storages);
            additional += 1;
        }
        drop(storages);

        // Then, add the state bits 
        for (_, column) in self.table.iter_mut() {
            column.states_mut().extend_with_flags(
                additional,
                StateFlags {
                    added: true,
                    modified: true,
                },
            );
        }

        // Allocate the entities then add them as well
        for _ in 0..additional {
            let linkings = EntityLinkings {
                mask: self.mask,
                index: self.len(),
            };
            let entity = entities.insert(linkings);
            self.entities.push(entity);
        }

        log::debug!(
            "Extended archetype {} with {} new elements",
            self.mask,
            additional
        );

        // Return the newly added entity IDs
        &self.entities[old_len..]
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        log::debug!(
            "Reserving {} additional elements for archetype {}",
            additional,
            self.mask
        );
        self.entities.reserve(additional);

        // Reserve more memory for the columns
        for (_, column) in self.table.iter_mut() {
            column.reserve(additional);
        }
    }

    // Shrink the memory allocation used by this archetype
    pub fn shrink_to_fit(&mut self) {
        self.entities.shrink_to_fit();
        self.table.shrink_to_fit();

        // Shrink the actual column allocation
        for (_, column) in self.table.iter_mut() {
            column.shrink_to_fit();
        }
    }

    // Get the number of entities that reference this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    // Check if the archetype is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    // Get the entity slice immutably
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // Get the unique archetype mask
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Try to get an immutable reference to a column of a specific component
    pub fn column<T: Component>(&self) -> Option<&Column<T>> { 
        let boxed = self.table.get(&mask::<T>())?;
        Some(boxed.as_any().downcast_ref::<Column<T>>().unwrap())
    }
    
    // Try to get a mutable reference to a column of a specific component
    pub fn column_mut<T: Component>(&mut self) -> Option<&mut Column<T>> { 
        let boxed = self.table.get_mut(&mask::<T>())?;
        Some(boxed.as_any_mut().downcast_mut::<Column<T>>().unwrap())
    }

    // Get the internal table immutably
    pub fn table(&self) -> &Table {
        &self.table
    }

    // Get the internal table mutably
    pub fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

// This will get two different archetypes using their masks
// This assumes that the archetypes exist already in the set, and that we are using different masks
fn split(
    set: &mut ArchetypeSet,
    mask1: Mask,
    mask2: Mask,
) -> (&mut Archetype, &mut Archetype) {
    assert_ne!(mask1, mask2);
    let a1 = set.get_mut(&mask1).unwrap() as *mut Archetype;
    let a2 = set.get_mut(&mask2).unwrap() as *mut Archetype;
    unsafe {
        let a1 = &mut *a1;
        let a2 = &mut *a2;
        (a1, a2)
    }
}

// Initialize a new archetype for when we add a bundle to an entity
fn init_archetype_added_bundle<B: Bundle>(
    archetypes: &MaskHashMap<Archetype>,
    old: Mask,
    new: Mask
) -> Archetype {
    let current = archetypes.get(&old).unwrap();
    let base_columns = current
        .table
        .iter()
        .map(|(mask, table)| (*mask, table.clone_default()));
    let mut columns = B::default_tables();
    columns.extend(base_columns);
    
    Archetype {
        mask: new,
        table: columns,
        entities: Default::default(),
    }
}

// Initialize a new archetype for when we remove a bundle from a bundle
fn init_archetype_removed_bundle<B: Bundle>(
    archetypes: &MaskHashMap<Archetype>,
    old: Mask,
    new: Mask
) -> Archetype {
    let current = archetypes.get(&old).unwrap();
    let columns = current
        .table
        .iter()
        .filter(|(mask, _)| new.contains(**mask))
        .filter(|(mask, _)| Mask::contains(&new, **mask))
        .map(|(mask, table)| (*mask, table.clone_default()));
    
    Archetype {
        mask: new,
        table: MaskHashMap::from_iter(columns),
        entities: Default::default(),
    }
}


// Add some new components onto an entity, forcing it to switch archetypes
pub(crate) fn add_bundle<B: Bundle>(
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
    let bundle_mask = B::reduce(|a, b| a | b);
    let old = entities[entity].mask;
    let new = entities[entity].mask | bundle_mask;

    // Nothing changed, don't execute
    if new == old {
        return None;
    }

    // Create the new target archetype if needed
    if !archetypes.contains_key(&new) {
        let arch = init_archetype_added_bundle::<B>(archetypes, old, new);
        archetypes.insert(new, arch);

        log::debug!(
            "Created new archetype with mask {} (added bundle)",
            new
        );
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.get(entity).unwrap();
    let index = linkings.index();

    // Move the components and states from one archetype to the other
    for (mask, input) in current.table.iter_mut() {
        let output = target.table.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

    // Add the extra components to the archetype
    let mut storages = B::prepare(target).unwrap();
    // TODO: Handle states within B::push??
    B::push(bundle, &mut storages);
    drop(storages);

    // Add the extra states as well
    for (_, output) in target
        .table_mut()
        .iter_mut()
        .filter(|(mask, _)| bundle_mask.contains(**mask))
    {
        output.states_mut().extend_with_flags(
            1,
            StateFlags {
                added: true,
                modified: true,
            },
        )
    }

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
pub(crate) fn remove_bundle<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
) -> Option<()> {
    assert!(
        B::is_valid(),
        "Bundle is not valid, check the bundle for component collisions"
    );

    // Get the old and new masks
    let old = entities[entity].mask;
    let bundle_mask = B::reduce(|a, b| a | b);
    let new = entities[entity].mask & !bundle_mask;

    // Check if we even have the bundle stored
    if !old.contains(bundle_mask) {
        return None;
    }

    // Create the new target archetype if needed
    if !archetypes.contains_key(&new) {
        let arch = init_archetype_removed_bundle::<B>(archetypes, old, new);
        archetypes.insert(new, arch);

        log::debug!(
            "Created new archetype with mask {} (removed bundle)",
            new
        );
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.get(entity)?;
    let index = linkings.index();

    // Move the components and states from one archetype to the other (flipped)
    for (mask, output) in target.table.iter_mut() {
        let input = current.table.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

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