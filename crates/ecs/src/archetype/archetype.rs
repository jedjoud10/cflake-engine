use crate::{
    entity::{Entity, EntityLinkings},
    mask, ArchetypeSet, Bundle, Component, ComponentColumn, EntitySet, Mask, MaskHashMap,
    QueryLayoutRef, StateColumn, StateFlags,
};
use std::{cell::RefCell, rc::Rc};

// We store two different column-major tables within the archetypes
pub type ComponentTable = MaskHashMap<Box<dyn ComponentColumn>>;
pub type StateTable = MaskHashMap<StateColumn>;

// An archetype is a special structure that contains multiple entities of the same layout
// Archetypes are used in archetypal ECSs to improve iteration and insertion/removal performance
pub struct Archetype {
    mask: Mask,
    components: ComponentTable,
    states: StateTable,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create a new archetype from a owned bundle accessor
    // This assumes that B is a valid bundle
    pub(crate) fn from_table_accessor<B: Bundle>() -> Self {
        let mask = B::reduce(|a, b| a | b);
        Self {
            mask,
            components: B::default_tables(),
            states: MaskHashMap::from_iter(mask.units().map(|mask| (mask, StateColumn::default()))),
            entities: Vec::new(),
        }
    }

    // Create the unit archetype that contains no columns and has a zeroed mask
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
            column.extend_with_flags(additional, StateFlags {
                added: true,
                removed: false,
                modified: true,
            });
        }
        
        // Add the storage bundles to their respective columns
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

        // Reserve more memory for the components columns
        for (_, column) in &mut self.components {
            column.reserve(additional);
        }
        
        // Reserve more memory for the state columns
        for (_, column) in &mut self.states {
            column.reserve(additional);
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
    pub fn components<T: Component>(&self) -> Option<&Vec<T>> {
        let boxed = &self.components.get(&mask::<T>())?;
        Some(boxed.as_any().downcast_ref().unwrap())
    }

    // Try to get a mutable reference to the table for a specific component
    pub(crate) fn components_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        let boxed = self.components.get_mut(&mask::<T>())?;
        Some(boxed.as_any_mut().downcast_mut().unwrap())
    }

    // Try to get an immutable reference to the state table for a specific component
    pub fn states<T: Component>(&self) -> Option<&StateColumn> {
        self.states.get(&mask::<T>())
    }

    // Try to get a mutable reference to the state table for a specific component
    pub(crate) fn states_mut<T: Component>(&mut self) -> Option<&mut StateColumn> {
        self.states.get_mut(&mask::<T>())
    }
    
    // Get the component table immutably
    pub fn component_table(&self) -> &ComponentTable {
        &self.components
    }
    
    // Get the component table mutably
    pub(crate) fn component_table_mut(&mut self) -> &mut ComponentTable {
        &mut self.components
    }

    // Get the state table immutably
    pub fn state_table(&self) -> &StateTable {
        &self.states
    }

    // Get the state table mutably
    pub fn state_table_mut(&mut self) -> &mut StateTable {
        &mut self.states
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

        // Remove the components from the columns
        for (_, column) in self.components.iter_mut() {
            column.swap_remove(index)
        }

        // Remove the states from the columns
        for (_, column) in self.states.iter_mut() {
            column.swap_remove(index);
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
    let additional = B::reduce(|a, b| a | b);
    let old = entities[entity].mask;
    let new = entities[entity].mask | additional;

    // Nothing changed, don't execute
    if new == old {
        return Some(());
    }

    // Create the new target archetype if needed
    if !archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let base = current
            .components
            .iter()
            .map(|(mask, table)| (*mask, table.clone_default()));
        let mut components = B::default_tables();
        components.extend(base);
        
        let base = current
            .states
            .iter()
            .map(|(mask, _)| (*mask, StateColumn::default()));
        let mask = B::reduce(|a, b| a | b);
        let mut states = MaskHashMap::from_iter(mask.units().map(|mask| (mask, StateColumn::default())));
        states.extend(base);

        let archetype = Archetype {
            mask: new,
            components,
            states,
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.get(entity).unwrap();
    let index = linkings.index();

    // Move the components from one archetype to the other
    for (mask, input) in current.components.iter_mut() {
        let output = target.components.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

    // Move the states from one archetype to the other
    for (mask, input) in current.states.iter_mut() {
        let output = target.states.get_mut(mask).unwrap();
        input.swap_remove_move(index, output);
    }

    // Add the extra components as well
    let mut storages = B::prepare(target).unwrap();
    B::push(&mut storages, bundle);
    drop(storages);

    // Add the extra states as well
    for (_, output) in target.state_table_mut().iter_mut().filter(|(mask, _)| additional.contains(**mask)) {
        output.extend_with_flags(1, StateFlags {
            added: true,
            modified: true,
            removed: false
        })
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
    if !archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let components = current
            .components
            .iter()
            .filter(|(mask, _)| new.contains(**mask))
            .filter(|(mask, _)| Mask::contains(&new, **mask))
            .map(|(mask, table)| (*mask, table.clone_default()));

        let states = current
            .states
            .iter()
            .filter(|(mask, _)| Mask::contains(&new, **mask))
            .filter(|(mask, _)| new.contains(**mask))
            .map(|(mask, _)| (*mask, StateColumn::default()));
        
        let archetype = Archetype {
            mask: new,
            components: MaskHashMap::from_iter(components),
            states: MaskHashMap::from_iter(states),
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.get(entity)?;
    let index = linkings.index();

    // Move the components from one archetype to the other (flipped)
    for (mask, output) in target.components.iter_mut() {
        let input = current.components.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

    // Move the states from one archetype to the other (flipped)
    for (mask, output) in target.states.iter_mut() {
        let input = current.states.get_mut(mask).unwrap();
        input.swap_remove_move(index, output);
    }

    // Create the return bundle
    let bundle = B::try_swap_remove(&mut current.components, index);

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