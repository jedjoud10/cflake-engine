use std::slice;

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

    // Move an entity from an archetype to another archetype
    pub(crate) fn move_entity(&mut self, old: Mask, new: Mask, entity: Entity, linkings: &mut EntityLinkings, extra: Vec<(Mask, Box<dyn Any>)>) {
        // Make sure archetypes masks are disjoint
        if old == new {
            return None;
        }

        // Check if the masks are valid
        if !self.archetypes.contains_key(&old) || !self.archetypes.contains_key(&new) {
            return None;
        }

        // A bit of unsafe code but this should technically still be safe
        let ptr1: *mut Archetype = self.archetypes.get_mut(&old).unwrap();
        let ptr2: *mut Archetype = self.archetypes.get_mut(&new).unwrap();
        let (new, old) = unsafe { (&mut *ptr1, &mut *ptr2) };
    
        // Move
        old.move_entity_to()
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
    // Insert multiple entities in batch. The entities must have the same component layout
    // This will panic if one of the entities contains a different component layout than the others
    // Will also panic if count is equal to 0
    pub fn insert_batch(&mut self, count: usize, mut function: impl FnMut(usize, Entity, &mut Linker)) -> &[Entity] {
        // Bruh count
        assert_ne!(count, 0, "Cannot insert a batch of 0 entities");

        // Add the first entity normally, so we can get the output archetype
        self.entities.reserve(count);
        self.states.extend_by(count, EntityState::new(true, true));
        let entity = self.insert(|entity, linker| function(0, entity, linker));

        // Archetype moment
        let archetype = self.archetypes.get_mut(&self.entities[entity].mask).unwrap();
        let start_index = archetype.entities.len();
        archetype.reserve(count);

        // If we only added one entity, return early
        if count == 1 {
            let elem = &archetype.entities[start_index - 1];
            return slice::from_ref(elem);
        }

        // Add the entities, and make sure that they all have the same layout
        for x in 1..count {
            // Create a strict linker, since we know the target archetype now
            let entity = self.entities.insert(EntityLinkings::default());
            let mut linker = Linker::new_strict(entity, archetype, &mut self.entities[entity]);
            function(x, entity, &mut linker);
            linker.apply();
        }
        &archetype.entities[start_index..]
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
