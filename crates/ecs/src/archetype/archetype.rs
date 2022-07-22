use crate::{
    entity::{Entity, EntityLinkings},
    mask, ArchetypeSet, Bundle, Component, ComponentTable, EntitySet, Mask, MaskMap, StateRow,
};
use std::{cell::RefCell, rc::Rc};

// TODO: Comment
pub struct Archetype {
    mask: Mask,
    tables: MaskMap<Box<dyn ComponentTable>>,
    states: Rc<RefCell<Vec<StateRow>>>,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create a new archetype from a owned bundle accessor
    // This assumes that B is a valid bundle
    pub(crate) fn from_table_accessor<B: Bundle>() -> Self {
        Self {
            mask: B::combined(),
            tables: B::default_tables(),
            states: Rc::new(RefCell::new(Vec::new())),
            entities: Vec::new(),
        }
    }

    // Create the unit archetype that contains no tables and has a zeroed mask
    pub(crate) fn empty() -> Self {
        Self {
            mask: Mask::zero(),
            tables: Default::default(),
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
    ) -> Vec<Entity> {
        assert!(B::is_valid());
        assert_eq!(B::combined(), self.mask);
        self.reserve(entities.len());
        let mut output = Vec::new();

        // Add the entities internally and externally
        for _ in 0..components.len() {
            let linkings = EntityLinkings {
                mask: self.mask,
                index: self.len(),
            };
            let entity = entities.insert(linkings);
            self.states
                .borrow_mut()
                .push(StateRow::new(self.mask, Mask::zero(), self.mask));
            self.entities.push(entity);
            output.push(entity)
        }

        // Add the storage bundles to their respective tables
        let mut storages = B::prepare(self).unwrap();
        for set in components {
            B::push(&mut storages, set);
        }

        // Return the newly added entity IDs
        output
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.states.borrow_mut().reserve(additional);

        for (_, table) in &mut self.tables {
            table.reserve(additional);
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

    // Get a copy of the archetype states
    pub fn states(&self) -> Rc<RefCell<Vec<StateRow>>> {
        self.states.clone()
    }

    // Try to get an immutable reference to the table for a specific component
    pub fn table<T: Component>(&self) -> Option<&Vec<T>> {
        let boxed = self.tables.get(&mask::<T>())?;
        Some(boxed.as_any().downcast_ref().unwrap())
    }

    // Try to get a mutable reference to the table for a specific component
    pub(crate) fn table_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        let boxed = self.tables.get_mut(&mask::<T>())?;
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
        for (_, table) in self.tables.iter_mut() {
            table.swap_remove(index)
        }

        // Remove the entity and get the entity that was swapped with it
        self.entities.swap_remove(index);
        self.states.borrow_mut().swap_remove(index);
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
// This assumes that the OwnedBundle type is valid for this use case
pub(crate) fn add_bundle_unchecked<B: Bundle>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    entities: &mut EntitySet,
    bundle: B,
) -> Option<()> {
    // Get the old and new masks
    let old = entities[entity].mask;
    let new = entities[entity].mask | B::combined();

    // Nothing changed, don't execute
    if new == old {
        return Some(());
    }

    // Create the new target archetype if needed
    if archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let tables = current
            .tables
            .iter()
            .map(|(mask, table)| (*mask, table.clone_default()));
        let archetype = Archetype {
            mask: new,
            tables: MaskMap::from_iter(tables),
            states: Default::default(),
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.remove(entity)?;
    let index = linkings.index();

    // Move the components from one archetype to the other
    for (mask, input) in current.tables.iter_mut() {
        let output = target.tables.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

    // Add the extra components as well
    let mut storages = B::prepare(target)?;
    B::push(&mut storages, bundle);
    drop(storages);

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    current.states.borrow_mut().swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target
        .states
        .borrow_mut()
        .push(StateRow::new(target.mask, Mask::zero(), target.mask));
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
    // Get the old and new masks
    let old = entities[entity].mask;
    let new = entities[entity].mask & !B::combined();

    // Create the new target archetype if needed
    if archetypes.contains_key(&new) {
        let current = archetypes.get_mut(&old).unwrap();
        let tables = current
            .tables
            .iter()
            .map(|(mask, table)| (*mask, table.clone_default()));
        let filtered = tables.filter(|(mask, _)| Mask::contains(&new, *mask));
        let archetype = Archetype {
            mask: new,
            tables: MaskMap::from_iter(filtered),
            states: Default::default(),
            entities: Default::default(),
        };
        archetypes.insert(new, archetype);
    }

    // Get the current and target archetypes that we will modify
    let (current, target) = split(archetypes, old, new);
    let linkings = entities.remove(entity)?;
    let index = linkings.index();

    // Move the components from one archetype to the other (swapped)
    for (mask, output) in target.tables.iter_mut() {
        let input = current.tables.get_mut(mask).unwrap();
        input.swap_remove_move(index, output.as_mut());
    }

    // Create the return bundle
    let bundle = B::try_swap_remove(&mut current.tables, index);

    // Handle swap-remove logic in the current archetype
    current.entities.swap_remove(index);
    current.states.borrow_mut().swap_remove(index);
    if let Some(entity) = current.entities.get(index).cloned() {
        let swapped = entities.get_mut(entity).unwrap();
        swapped.index = index;
    }

    // Insert the new entity in the target archetype
    let linkings = entities.get_mut(entity).unwrap();
    target
        .states
        .borrow_mut()
        .push(StateRow::new(target.mask, B::combined(), target.mask));
    target.entities.push(entity);
    linkings.index = target.len() - 1;
    linkings.mask = target.mask;

    bundle
}
