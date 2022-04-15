use std::{slice, any::Any};

use slotmap::SlotMap;

use crate::{
    entity::Entity, registry, Archetype, Component, EntityLinkings, EntityState, EntityStateSet, Entry, LinkModifier, Linker, Mask, MaskMap, QueryCache, QueryError, QueryIter,
    QueryLayout, StorageVec,
};

// Type aliases for simpler names
pub type EntitySet = SlotMap<Entity, EntityLinkings>;
pub type ArchetypeSet = MaskMap<Archetype>;
pub(crate) type UniqueStoragesSet = MaskMap<Box<dyn StorageVec>>;

#[derive(Default)]
pub struct EcsManager {
    // Hmmm
    pub(crate) entities: EntitySet,
    pub(crate) archetypes: ArchetypeSet,
    pub(crate) uniques: UniqueStoragesSet,

    // Others
    states: EntityStateSet,
    count: u64,
    cache: QueryCache,
}

impl EcsManager {
    // Register a component to be used
    pub fn register<T: Component>(&mut self) {
        registry::register::<T>();
    }

    // Return Some(Entity) if the entity is valid. Otherwise, return None
    pub fn validate(&self, entity: Entity) -> Option<Entity> {
        self.states.get(entity).unwrap().valid().then(|| entity)
    }

    // Prepare the Ecs Manager for one execution
    pub fn prepare(&mut self) {
        // Reset the archetype component mutation bits
        for (_, archetype) in self.archetypes.iter_mut() {
            archetype.prepare(self.count)
        }

        // Iteration counter that keeps track how many times we've run the ECS system
        self.count += 1;
    }

    // Modify an entity's component layout
    pub fn modify(&mut self, entity: Entity, function: impl FnOnce(Entity, &mut LinkModifier)) -> Option<()> {
        // Keep a copy of the linkings before we do anything
        let mut copied = *self.entities.get(self.validate(entity)?)?;

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Apply the changes
        linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;
        Some(())
    }

    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<Entry> {
        Entry::new(self, self.validate(entity)?)
    }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert(&mut self, function: impl FnOnce(Entity, &mut Linker)) -> Entity {
        // Check if we expanded the slot map
        let entity = self.entities.insert(EntityLinkings::default());

        // Create a linker, so we can insert components and link them to the entity
        let mut linker = Linker::new_simple(self, entity);
        function(entity, &mut linker);

        // Apply the changes (adds it to the archetype)
        linker.apply();

        // Set the new entity's state
        self.states.extend_if_needed(entity);
        self.states.set(entity, EntityState::new(true, true));

        entity
    }

    // Remove an entity from the world, instantly
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Get the archetype and the linkings, and check if the latter is valid        ;
        let linkings = self.entities.get_mut(self.validate(entity)?)?;
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();

        // Remove
        archetype.remove(linkings.bundle);
        Some(())
    }

    // Get a component query that we will use to read/write to certain components
    pub fn query<'a, Layout: QueryLayout<'a>>(&'a mut self) -> Result<QueryIter<'a, Layout>, QueryError> {
        self.cache.update(&mut self.archetypes);
        QueryIter::new(&self.cache)
    }
}
