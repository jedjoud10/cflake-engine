use super::{ComponentUnlinkGroup, Entity, EntityKey};
use crate::{component::ComponentSet, system::SystemSet, utils::EntityError};
use getset::Getters;
use slotmap::SlotMap;

// Entity set
#[derive(Getters)]
pub struct EntitySet {
    #[getset(get = "pub")]
    pub(crate) inner: SlotMap<EntityKey, Entity>,
}

impl Default for EntitySet {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl EntitySet {
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
    pub fn remove<World>(&mut self, key: EntityKey, components: &mut ComponentSet, systems: &mut SystemSet<World>) -> Result<(), EntityError> {
        let entity = self.inner.get(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))?;
        let group = ComponentUnlinkGroup::unlink_all_from_entity(entity);
        // Unlink all of the entity's components
        components.unlink(key, self, systems, group).unwrap();
        // Invalidate the entity
        let _entity = self.inner.remove(key).ok_or_else(|| EntityError::new("Could not find entity!".to_string(), key))?;
        Ok(())
    }
}
