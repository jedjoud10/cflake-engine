use crate::{
    archetype::{ArchetypeSet, UniqueComponentStoragesHashMap},
    entity::{Entity, EntitySet},
    EntityEntry, EntityLinkings, LinkModifier, Linker, ProfiledEventTiming, QueryCache, QueryIter, QueryLayout,
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
        let mut linker = Linker::new(self, entity);
        function(entity, &mut linker);

        // Apply the changes (adds it to the archetype)
        let linkings = linker.apply();

        // Update the cache of the receiving archetype
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        self.cache.update(archetype);

        entity
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
    pub fn query<'a, Layout: QueryLayout<'a>>(&'a mut self) -> QueryIter<'a, Layout> {
        QueryIter::new(&self.cache)
    }
}
