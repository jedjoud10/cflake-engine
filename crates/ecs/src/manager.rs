use std::{cell::UnsafeCell, mem::size_of};

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator,
};

use crate::{
    archetype::{
        Archetype, ArchetypeError, ArchetypeSet, ComponentStoragesHashMap,
        UniqueComponentStoragesHashMap,
    },
    entity::{Entity, EntityLinkings, EntitySet},
    prelude::{registry, Mask, SystemSet, LinkModifier, EntityEntry},
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
        for (_, archetype) in self.archetypes.iter_mut() {
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

    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<EntityEntry> {
        EntityEntry::new(self, entity)
    }

    // Insert an empty entity into the manager
    pub fn insert(&mut self) -> Entity { self.entities.insert(None) }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert_with(&mut self, function: impl FnOnce(Entity, &mut LinkModifier)) -> Entity {
        let entity = self.insert();
        // Create a link modifier, so we can insert components into the archetypes
        let mut modifier = LinkModifier::new(self, entity);
        function(entity, &mut modifier);
        // Apply the changes
        modifier.apply();
        entity
    }

    // Remove an entity from the world
    // This will set it's entity state to PendingForRemoval, since we actually remove the entity next iteration
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Check if the entity is valid
        let maybe = self.entities.get_mut(entity).and_then(|x| x.as_mut());
        let linkings = if let Some(linkings) = maybe {
            linkings
        } else {
            return None;
        };

        // Check if the linking is valid (the entity is not pending for removal)
        if !linkings.is_valid() { return None }

        // Get the arhcetype and commence the bundle's components deletion process
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        archetype.add_pending_for_removal(linkings.bundle);
        linkings.mask = Default::default();

        Some(())
    }
    /*
    let linkings = self.entities.remove(entity)?;

    // In case the entity actually had components
    if let Some(linkings) = linkings {
        // Simply remove the components that were linked to this entity
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        archetype.remove(entity, linkings);
    }

    // Result
    Some(())
    */

    // Add a new system (stored as an event) into the manager
    pub fn system<World>(&mut self, evn: fn(&mut World), systems: &mut SystemSet<World>) {
        // Borrow since it's stored in an RC
        let mut borrow = systems.inner.borrow_mut();
        borrow.push(evn);
    }
}
