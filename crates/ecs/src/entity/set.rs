use slotmap::SlotMap;

use super::{Entity, EntityKey, EntityLinkings};

#[derive(Default)]
pub struct EntitySet {
    // Entities -> (Archetype Index, Component Bundle Index)
    entities: SlotMap<EntityKey, Option<EntityLinkings>>,
}

impl EntitySet {
    // Insert an entity into the set
    pub fn insert(&mut self, linkings: Option<EntityLinkings>) -> Entity {
        Entity(self.entities.insert(linkings))
    }

    // Get and get mut
    pub fn get(&self, entity: Entity) -> Option<&EntityLinkings> {
        self.entities.get(entity.0).map(|x| x.as_ref()).flatten()
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLinkings> {
        self.entities.get_mut(entity.0).map(|x| x.as_mut()).flatten()
    }
}
