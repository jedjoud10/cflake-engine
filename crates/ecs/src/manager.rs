use std::collections::hash_map::Entry;

use slotmap::SlotMap;

use crate::{
    entity::Entity, registry, Archetype, Component, EntityEntry, EntityLinkings, EntityStateSet, LinkModifier, Linker, Mask, MaskMap, QueryCache, QueryError, QueryIter,
    QueryLayout, StorageVec, EntityState, ArchetypalState,
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

    // Get two archetypes at the same time
    pub fn get_disjoint_archetypes(&mut self, m1: Mask, m2: Mask) -> Option<(&mut Archetype, &mut Archetype)> {
        // Make sure archetypes masks are disjoint
        if m1 == m2 {
            return None;
        }

        // Check if the masks are valid
        if !self.archetypes.contains_key(&m1) || self.archetypes.contains_key(&m2) {
            return None
        }

        // A bit of unsafe code but this should technically still be safe
        let ptr1: *mut Archetype = self.archetypes.get_mut(&m1).unwrap();
        let ptr2: *mut Archetype = self.archetypes.get_mut(&m2).unwrap();
        let borrowed = unsafe { (&mut *ptr1, &mut *ptr2) };
        Some(borrowed)
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
        let mut copied = *self.entities.get(entity)?;
        self.states.get(entity).unwrap().is_valid().then(|| ())?;

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Apply the changes
        linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;
        Some(())
    }

    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<EntityEntry> {
        self.states.get(entity).unwrap().is_valid().then(|| ())?;
        EntityEntry::new(self, entity)
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
        self.states.set(EntityState::DEFAULT_STATE, entity);

        entity
    }
    // Insert multiple entities in batch. The entities must have the same component layout
    // This will panic if one of the entities contains a different component layout than the others
    pub fn insert_batch(&mut self, count: usize, mut function: impl FnMut(usize, Entity, &mut Linker)) -> &[Entity] {
        // Add the first entity normally, so we can get the output archetype
        self.entities.reserve(count);
        self.states.extend_by(count, EntityState::DEFAULT_STATE);
        let entity = self.insert(|entity, linker| function(0, entity, linker));

        // Archetype moment
        let archetype = self.archetypes.get_mut(&self.entities[entity].mask).unwrap();
        let start_index = archetype.entities.len();
        archetype.reserve(count - 1);

        // Add the entities, and make sure that they all have the same layout
        for x in 1..count {
            // Create a strict linker, since we know the target archetype now
            let entity = self.entities.insert(EntityLinkings::default());   
            let mut linker = Linker::new_strict(entity, archetype, &mut self.entities[entity]);
            function(x, entity, &mut linker);
            self.states.get(entity).unwrap();
            linker.apply();
        }
        &archetype.entities[start_index..]
    }

    // Remove an entity from the world
    // This will set it's entity state to PendingForRemoval, since we actually remove the entity next iteration
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Get the archetype and the linkings, and check if the latter is valid
        let linkings = self.entities.get_mut(entity)?;
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        self.states.get(entity).unwrap().is_valid().then(|| ())?;

        // Apply the "pending for removal" state
        archetype.add_pending_for_removal(linkings.bundle);
        linkings.mask = Default::default();

        Some(())
    }

    // Get a component query that we will use to read/write to certain components
    pub fn query<'a, Layout: QueryLayout<'a>>(&'a mut self) -> Result<QueryIter<'a, Layout>, QueryError> {
        self.cache.update(&mut self.archetypes);
        QueryIter::new(&self.cache)
    }
}
