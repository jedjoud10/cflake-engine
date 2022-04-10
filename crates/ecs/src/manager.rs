use crate::{
    archetype::{ArchetypeSet, UniqueComponentStoragesHashMap},
    entity::{Entity, EntitySet},
    registry, Component, EntityEntry, EntityLinkings, LinkModifier, Linker, QueryCache, QueryError, QueryIter, QueryLayout,
};

// Manages ECS logic
#[derive(Default)]
pub struct EcsManager {
    // Entities
    pub(crate) entities: EntitySet,

    // Archetypes
    pub(crate) archetypes: ArchetypeSet,

    // Iteration count
    count: u64,

    // Unique component storages
    pub(crate) uniques: UniqueComponentStoragesHashMap,

    // Query cache for bRRRR type performance
    cache: QueryCache,
}

impl EcsManager {
    // Register a component
    pub fn register<T: Component>(&mut self) {
        registry::register::<T>();
    }

    // Check if an entity is valid
    pub fn is_valid(&self, entity: Entity) -> Option<bool> {
        let linkings = self.entities.get(entity)?;
        let archetype = self.archetypes.get(&linkings.mask).unwrap();
        Some(archetype.is_valid(linkings.bundle))
    }

    // Force a cache update
    pub fn update_cache(&mut self) {
        self.archetypes.iter_mut().for_each(|x| self.cache.update(x));
    }

    // Prepare the Ecs Manager for one execution
    pub fn prepare(&mut self) {
        // Reset the archetype component mutation bits
        for archetype in self.archetypes.iter_mut() {
            archetype.prepare(self.count)
        }

        // Iteration counter that keeps track how many times we've run the ECS system
        self.count += 1;
    }

    // Modify an entity's component layout
    pub fn modify(&mut self, entity: Entity, function: impl FnOnce(Entity, &mut LinkModifier)) -> Option<()> {
        // Keep a copy of the linkings before we do anything
        let mut copied = *self.entities.get(entity)?;
        if !self.is_valid(entity).unwrap() {
            return None;
        }

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Apply the changes
        let (old, new) = linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;

        // Update the old archetype and new archetype
        let (old, new) = self.archetypes.get_two_mut(old, new).unwrap();
        self.cache.update(old);
        self.cache.update(new);
        Some(())
    }

    // Get an entity entry
    pub fn entry(&mut self, entity: Entity) -> Option<EntityEntry> {
        EntityEntry::new(self, entity)
    }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert(&mut self, function: impl FnOnce(Entity, &mut Linker)) -> Entity {
        let entity = self.entities.insert(EntityLinkings::default());

        // Create a linker, so we can insert components and link them to the entity
        let mut linker = Linker::new_simple(self, entity);
        function(entity, &mut linker);

        // Apply the changes (adds it to the archetype)
        let linkings = linker.apply();

        // Update the cache of the receiving archetype
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        self.cache.update(archetype);

        entity
    }
    // Insert multiple entities in batch. The entities must have the same component layout
    pub fn insert_batch(&mut self, count: usize, mut function: impl FnMut(usize, Entity, &mut Linker)) -> Option<&[Entity]> {
        // Add the first entity normally, so we can get the output archetype
        let i = std::time::Instant::now();
        let entity = self.insert(|entity, linker| {
            function(0, entity, linker)
        });
        dbg!(i.elapsed());

        let archetype = self.archetypes.get_mut(&self.entities[entity].mask).unwrap();
        let start_index = archetype.entities.len();

        // Make sure the archetype has enough space allocated, so it won't reallocate as many times
        archetype.reserve(count-1);

        // Add the entities, and make sure that they all have the same layout
        for x in 0..(count-1) {
            // Create a strict linker, since we know the target archetype now
            let mut linker = Linker::new_strict(archetype, &mut self.entities[entity], entity);
            function(x+1, entity, &mut linker);
            linker.apply();
        }        
        // Update the archetype cache at the end
        self.cache.update(archetype);

        Some(&archetype.entities[start_index..])
    }

    // Remove an entity from the world
    // This will set it's entity state to PendingForRemoval, since we actually remove the entity next iteration
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Get the archetype and the linkings, and check if the latter is valid
        let linkings = self.entities.get_mut(entity)?;
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        if !archetype.is_valid(linkings.bundle) {
            return None;
        }

        // Apply the "pending for removal" state
        archetype.add_pending_for_removal(linkings.bundle);
        linkings.mask = Default::default();

        // Zad
        self.update_cache();

        Some(())
    }

    // Get a component query that we will use to read/write to certain components
    pub fn query<'a, Layout: QueryLayout<'a>>(&'a mut self) -> Result<QueryIter<'a, Layout>, QueryError> {
        QueryIter::new(&self.cache)
    }
}
