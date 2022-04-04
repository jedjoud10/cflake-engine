use crate::{
    archetype::{ArchetypeSet, UniqueComponentStoragesHashMap},
    entity::{Entity, EntitySet}, LinkModifier, Linker, SystemSet,
};

// Manages ECS logic
#[derive(Default)]
pub struct EcsManager {
    // Entities
    pub(crate) entities: EntitySet,

    // Archetypes
    pub(crate) archetypes: ArchetypeSet,

    // Unique component storages
    pub(crate) uniques: UniqueComponentStoragesHashMap,
}

impl EcsManager {
    // Prepare the Ecs Manager for one execution
    pub fn prepare(&mut self) {
        // Reset the archetype component mutation bits
        for archetype in self.archetypes.iter_mut() {
            archetype.prepare()
        }
    }

    // Execute the systems in sequence
    pub fn execute<World>(world: &mut World, systems: SystemSet<World>) {
        let borrowed = systems.inner.borrow();
        for event in borrowed.as_slice() {
            // Execute the system
            event(world)
        }
    }

    // Modify an entity's component layout
    pub fn modify(&mut self, entity: Entity, function: impl FnOnce(Entity, &mut LinkModifier)) -> Option<()> {
        // Just to check
        if !self.entities.contains_key(entity) {
            return None;
        }

        // Keep a copy of the linkings before we do anything
        let mut copied = *self.entities.get(entity)?;

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity);
        function(entity, &mut linker);

        // Apply the changes
        linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;
        Some(())
    }

    /*
    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<EntityEntry> {
        EntityEntry::new(self, entity)
    }
    */

    // Insert an empty entity into the manager
    pub fn insert(&mut self) -> Entity {
        self.entities.insert(None)
    }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert_with(&mut self, function: impl FnOnce(Entity, &mut Linker)) -> Entity {
        let entity = self.insert();
        // Create a linker, so we can insert components and link them to the entity
        let mut linker = Linker::new(self, entity);
        function(entity, &mut linker);
        // Apply the changes
        linker.apply();
        entity
    }

    // Remove an entity from the world
    // This will set it's entity state to PendingForRemoval, since we actually remove the entity next iteration
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Check if the entity is valid
        let maybe = self.entities.get_mut(entity).and_then(|x| x.as_mut());
        let linkings = maybe?;

        // Check if the linking is valid (the entity is not pending for removal)
        if !linkings.is_valid() {
            return None;
        }

        // Get the arhcetype and commence the bundle's components deletion process
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        archetype.add_pending_for_removal(linkings.bundle);
        linkings.mask = Default::default();

        Some(())
    }
}
