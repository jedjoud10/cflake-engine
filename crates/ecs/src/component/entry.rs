use std::marker::PhantomData;

use crate::{archetype::ArchetypeId, entity::Entity, manager::EcsManager};

use super::{Component, ComponentDeltas, ComponentQuery, QueryError};

// A guarded entity entry that we can fetch from the Ecs Manager that we can use to read/write to specific components on an entity
pub struct EntityEntry<'a> {
    // Internal query
    query: ComponentQuery,
    _phantom: PhantomData<&'a mut EcsManager>,
}

impl<'a> EntityEntry<'a> {
    // Create a new guarded entry using an ecs manager and some extra data
    pub(crate) fn new(manager: &'a mut EcsManager, bitmask: u64, entity: Entity, bundle: usize, archetype: ArchetypeId) -> Self {
        unsafe {
            Self {
                query: ComponentQuery::new(&manager.archetypes, bitmask, entity, bundle, archetype),
                _phantom: Default::default(),
            }
        }
    }
    // Get the component deltas
    pub fn deltas<T: Component>(&self) -> Result<ComponentDeltas, QueryError> {
        self.query.deltas::<T>()
    }
    // Get a component immutably
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        self.query.get()
    }
    // Get a component mutably
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        self.query.get_mut()
    }
}
