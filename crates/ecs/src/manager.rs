use slotmap::SlotMap;
use time::Time;
use world::{Events, Init, Resource, Stage, Update, World};

use crate::{
    entity::Entity, query, Archetype, ComponentTable,
    EntityLinkings, EntryRef, Evaluate, LinkError, Mask, MaskMap, EntryMut, OwnedBundle, QueryLayout,
    ViewLayout,
};

pub type EntitySet = SlotMap<Entity, EntityLinkings>;
pub type ArchetypeSet = MaskMap<Archetype>;

// TODO: Find a better name for this bozo
#[derive(Resource)]
pub struct EcsManager {
    // Entities are just objects that contain an ID and some component masks
    // Entities are linked to multiple components, but they don't store the component data by themselves
    pub(crate) entities: EntitySet,

    // Archetypes are a subset of entities that all share the same component mask
    // We use an archetypal ECS because it is a bit more efficient when iterating through components, though it is slower when modifying entity component layouts
    pub(crate) archetypes: ArchetypeSet,
}

impl Default for EcsManager {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            archetypes: MaskMap::from_iter(std::iter::once((Mask::zero(), Archetype::empty()))),
        }
    }
}

impl EcsManager {
    // Spawn an entity with specific components
    pub fn insert<B: for<'a> OwnedBundle<'a>>(&mut self, components: B) -> Entity {
        self.insert_with::<B>(|_| components)
    }

    // Spawn an entity using a specific callback
    pub fn insert_with<B: for<'a> OwnedBundle<'a>>(&mut self, callback: impl FnOnce(Entity) -> B) -> Entity {
        self.insert_from_iter_with(1, |entity, _| callback(entity))[0]
    }

    // Spawn a batch of entities with specific components
    pub fn insert_from_iter<B: for<'a> OwnedBundle<'a>>(&mut self, iter: impl IntoIterator<Item = B>) -> Vec<Entity> {
        let mut vec = iter.into_iter().collect::<Vec<B>>();
        // TODO: benchmark and optimize if needed
        vec.reverse();

        self.insert_from_iter_with(vec.len(), |_, _| vec.pop().unwrap())
    }

    // Spawn a batch of entities with specific componnets by calling a callback for each one
    pub fn insert_from_iter_with<B: for<'a> OwnedBundle<'a>>(&mut self, count: usize, callback: impl FnOnce(Entity, usize) -> B) -> Vec<Entity> {
        todo!()
    }

    // Remove an entity, and discard it's components
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        todo!()
    }

    // Remove an entity, and fetch it's removed components as a new bundle
    pub fn remove_then<B: for<'a> OwnedBundle<'a>>(&mut self, entity: Entity) -> Option<B> {
        todo!()
    }

    // Remove multiple entities, and discard their components
    pub fn remove_from_iter(&mut self, iter: impl IntoIterator<Item = Entity>) -> Option<()> {
        todo!()
    }

    // Remove multiple entities, and fetch their removed components as new bundles
    pub fn remove_from_iter_then<B: for<'a> OwnedBundle<'a>>(&mut self, iter: impl IntoIterator<Item = Entity>) -> Option<B> {
        todo!()
    }

    // Get the immutable entity entry for a specific entity
    pub fn entry(&self, entity: Entity) -> Option<EntryRef> {
        EntryRef::new(self, entity)
    }

    // Get the mutable entity entry for a specific entity
    pub fn entry_mut(&mut self, entity: Entity) -> Option<EntryMut> {
        EntryMut::new(self, entity)
    }

    // Get a immutable reference to the archetype set
    pub fn archetypes(&self) -> &ArchetypeSet {
        &self.archetypes
    }
    
    // Get a mutable reference to the archetype set
    pub fn archetypes_mut(&mut self) -> &mut ArchetypeSet {
        &mut self.archetypes
    }
    
    // Get an immutable reference to the entity set
    pub fn entities(&self) -> &EntitySet {
        &self.entities
    }
    
    // Get a mutable reference to the entity set
    pub fn entities_mut(&mut self) -> &mut EntitySet {
        &mut self.entities
    }

    // Create a new mutable query iterator    

    // Create a new mutable query iterator with a filter
    
    // Create a new immutable query iterator

    // Create a new immutable query iterator with a filter
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame (except the first frame)
pub fn system(events: &mut Events) {
    /*
    // Late update event that will cleanup the ECS manager states
    fn cleanup(world: &mut World) {
        let (ecs, _time) = world.get_mut::<(&mut EcsManager, &Time)>().unwrap();

        // Clear all the archetype states that were set last frame
        for (_, archetype) in ecs.archetypes() {
            archetype.states().reset();
        }
    }

    // Init event that will insert the ECS resource
    fn init(world: &mut World) {
        world.insert(EcsManager::default());
    }

    // Register the events
    events
        .registry::<Init>()
        .insert_with(init, Stage::new("ecs insert").before("user"))
        .unwrap();
    events
        .registry::<Update>()
        .insert_with(
            cleanup,
            Stage::new("ecs cleanup")
                .after("time update")
                .after("post user"),
        )
        .unwrap();
    */
}
