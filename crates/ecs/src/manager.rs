use crate::{
    archetype::{Archetype, ArchetypeError, ArchetypeId, ArchetypeSet, ArchetypeComponentInserter},
    component::{Component, ComponentLayout, ComponentQuery, EntityEntry, QueryError},
    entity::{Entity, EntityLinkings, EntitySet, EntityModifier},
    prelude::SystemSet,
};

// Manages ECS logic
#[derive(Default)]
pub struct EcsManager {
    // Entities
    pub entities: EntitySet,

    // Archetypes
    pub archetypes: ArchetypeSet,
}

impl EcsManager {
    // Prepare the Ecs Manager for one execution
    pub fn prepare<World>(&mut self) {
        // Reset the archetype component flags
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

    // Registers a new archetype. This becomes a no op if the archetype already exists
    pub fn register(&mut self, layout: ComponentLayout) -> ArchetypeId {
        self.archetypes.register(layout)
    }

    // Insert an empty entity into the manager
    pub fn insert(&mut self) -> Entity {
        self.entities.insert(None)
    }
    // Insert an entity into the manager with specific components
    pub fn insert_with(&mut self, id: ArchetypeId, callback: impl FnOnce(&mut ArchetypeComponentInserter)) -> Result<Entity, ArchetypeError> {
        // Get the correct archetype first
        let archetype = self.archetypes.get_mut(id).ok_or(ArchetypeError::NotFound)?;

        // Add the entity with an empty bundle index
        let entity = self.entities.insert(Some(EntityLinkings { archetype: id, bundle: 0 }));

        // Get the linkings and insert the components into the archetype
        let linkings = self.entities.get_mut(entity).unwrap();
        archetype.insert_with(callback, linkings, entity)?;
        Ok(entity)
    }

    // Get an entity entry using a specific entity ID
    pub fn entry<'a>(&'a mut self, entity: Entity, layout: ComponentLayout) -> EntityEntry<'a> {
        // Get the archetype ID and bundle index
        let EntityLinkings { archetype, bundle } = *self.entities.get(entity).unwrap();
        EntityEntry::new(self, layout.mask, entity, bundle, archetype)
    }
    // Query some linked components using the specific layout
    pub fn query(&mut self, layout: ComponentLayout) -> Vec<ComponentQuery> {
        // Loop through each archetype that satisfies the layout and extend the component queries
        let mut queries = Vec::<ComponentQuery>::new();
        for (id, archetype) in self.archetypes.iter() {
            // Convert the archetype ID into a bitmask
            let mask = id.0;

            // Check if it satisfies the field
            if (mask & layout.mask) == layout.mask {
                // Loop through each entity that is stored in the archetype
                queries.extend(
                    archetype
                        .entities()
                        .iter()
                        .enumerate()
                        .map(|(bundle, entity)| unsafe { ComponentQuery::new(&self.archetypes, layout.mask, *entity, bundle, *id) }),
                );
            }
        }
        // Return the queries
        queries
    }

    // Modify an entity by adding/removing components from it. This will move the entity from an archetype to another archetype, so it is not recomended to use it
    pub fn modify<'a>(&'a mut self, entity: Entity) -> EntityModifier<'a> {
        todo!()
    }

    // Add a new system (stored as an event) into the manager
    pub fn system<World>(&mut self, evn: fn(&mut World), systems: &mut SystemSet<World>) {
        // Borrow since it's stored in an RC
        let mut borrow = systems.inner.borrow_mut();
        borrow.push(evn);
    }
}
