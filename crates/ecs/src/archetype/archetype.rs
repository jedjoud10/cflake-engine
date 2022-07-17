use crate::{
    entity::{Entity, EntityLinkings},
    mask, registry, ArchetypeSet, Component, ComponentTable, EntitySet, Mask, MaskMap, OwnedBundle,
    StateRow, States,
};
use std::any::Any;

// TODO: Comment
pub struct Archetype {
    mask: Mask,
    tables: MaskMap<Box<dyn ComponentTable>>,
    states: States,
    entities: Vec<Entity>,
}

impl Archetype {
    // Create the unit archetype that contains no tables and has a zeroed mask
    pub(crate) fn empty() -> Self {
        Self {
            mask: Mask::zero(),
            tables: Default::default(),
            states: States::default(),
            entities: Default::default(),
        }
    }

    // Add multiple entities into the archetype with their corresponding owned components
    // The layout mask for "B" must be equal to the layout mask that this archetype contains
    pub(crate) fn extend_from_slice<B: for<'a> OwnedBundle<'a>>(
        &mut self,
        entities: Vec<(Entity, &mut EntityLinkings)>,
        components: Vec<B>,
    ) {
        assert!(B::is_valid());
        assert_eq!(entities.len(), components.len());
        assert_eq!(B::combined(), self.mask);

        self.reserve(entities.len());

        for (entity, linkings) in entities {
            self.states.push(StateRow::new(self.mask));
            self.entities.push(entity);
            linkings.index = self.len() - 1;
            linkings.mask = self.mask;
        }
        
        let mut storages = B::fetch(self);
        for set in components {
            B::push(&mut storages, set);
        }
    }

    // Reserve enough memory space to be able to fit all the new entities in one allocation
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.states.reserve(additional);

        for (_, table) in &mut self.tables {
            table.reserve(additional);
        }
    }

    // Get the number of entities that reference this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    // Get a list of the entities that are stored within this archetype
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // Get the unique archetype mask
    pub fn mask(&self) -> Mask {
        self.mask
    }

    // Get the current component states immutably
    pub fn states(&self) -> &States {
        &self.states
    }

    // Try to get an immutable reference to the table for a specific component
    pub fn table<T: Component>(&self) -> Option<&Vec<T>> {
        let boxed = self.tables.get(&mask::<T>())?;
        Some(boxed.as_any().downcast_ref().unwrap())
    }

    // Try to get a mutable reference to the table for a specific component
    pub fn table_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
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
        for (_, storages) in self.tables.iter_mut() {
            storages.swap_remove(index)
        }

        // Remove the entity and get the entity that was swapped with it
        self.entities.swap_remove(index);
        self.states.swap_remove(index);
        let entity = self.entities.get(index).cloned();

        // Swap might've failed if we swapped with the last element in the vector
        if let Some(entity) = entity {
            let swapped = entities.get_mut(entity).unwrap();
            swapped.index = index;
        }

        Some(linkings)
    }
    /*
    // Move an entity from an archetype to another archetype, whilst adding extra components to the entity
    pub(crate) fn move_entity(
        archetypes: &mut ArchetypeSet,
        entities: &mut EntitySet,
        old: Mask,
        new: Mask,
        entity: Entity,
        linkings: &mut EntityLinkings,
        extra: Vec<(Mask, Box<dyn Any>)>,
    ) {
        // Remove the entity (this might fail in the case of the default empty archetype)
        let mut removed = (old != Mask::zero())
            .then(|| Archetype::remove(archetypes, entities, entity, old))
            .unwrap_or_default();

        // Combine the removed components with the extra components
        removed.extend(extra);

        // And insert into the new archetype
        let new = archetypes.get_mut(&new).unwrap();
        new.push(entity, linkings, removed);
    }
    */
}

// Add some new components onto an entity, forcing it to switch archetypes
// This assumes that the OwnedBundle type is valid for this use case
pub(crate) fn add_bundle_unchecked<B: for<'a> OwnedBundle<'a>>(
    archetypes: &mut ArchetypeSet,
    entity: Entity,
    linkings: &mut EntityLinkings,
) {
    let old_mask = linkings.mask;
    let new_mask = linkings.mask | B::combined();
    None
}

// Remove some old components from an entity, forcing it to switch archetypes
// This assumes that the OwnedBundle type is valid for this use case
pub(crate) fn remove_bundle_unchecked<B: for<'a> OwnedBundle<'a>>(
    archetypes: &mut ArchetypeSet,
    entities: &mut EntitySet,
    entity: Entity,
) -> Option<B> {
    None
}
