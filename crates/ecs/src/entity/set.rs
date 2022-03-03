use std::marker::PhantomData;
use slotmap::SlotMap;
use crate::{utils::EntityError, component::ComponentSet, system::SystemSet};
use super::{EntityKey, Entity, ComponentUnlinkGroup};
use getset::Getters;

// Entity set
#[derive(Getters)]
pub struct EntitySet<World> {
    #[getset(get = "pub")]
    pub(crate) inner: SlotMap<EntityKey, Entity>,
    pub(crate) _phantom: PhantomData<World>,
}

impl<World> Default for EntitySet<World> {
    fn default() -> Self {
        Self { 
            inner: Default::default(),
            _phantom: Default::default()
        }
    }
}
 
impl<World> EntitySet<World> {
    // Get an entity
    pub fn get(&self, key: EntityKey) -> Result<&Entity, EntityError> {
        self.inner.get(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))
    }
    // Get a mutable entity
    pub fn get_mut(&mut self, key: EntityKey) -> Result<&mut Entity, EntityError> {
        self.inner.get_mut(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))
    }
    // Add an entity to the manager, and automatically link it's components
    pub fn add(&mut self, mut entity: Entity) -> Result<EntityKey, EntityError> {
        // Get the key
        let key = self.inner.insert_with_key(|key| {
            entity.key = key;
            entity
        });
        Ok(key)
    }
    // Remove an entity, but keep it's components alive until all systems have been notified
    pub fn remove(&mut self, key: EntityKey, components: &mut ComponentSet<World>, systems: &mut SystemSet<World>) -> Result<(), EntityError> {
        let entity = self.inner.get(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))?;
        let group = ComponentUnlinkGroup::unlink_all_from_entity(entity);
        // Unlink all of the entity's components
        components.unlink(key, self, systems, group).unwrap();
        // Invalidate the entity
        let _entity = self.inner.remove(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))?;
        Ok(())
    }
}