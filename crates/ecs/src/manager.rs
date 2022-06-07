use resources::Resource;
use slotmap::SlotMap;

use crate::{
    entity::Entity, filtered, query, Archetype, EntityLinkings, Entry, Evaluate, LinkModifier,
    Mask, MaskMap, QueryLayout, StorageVec,
};

// Type aliases because I have gone insane
pub type EntitySet = SlotMap<Entity, EntityLinkings>;
pub type ArchetypeSet = MaskMap<Archetype>;
pub(crate) type UniqueStoragesSet = MaskMap<Box<dyn StorageVec>>;

pub struct EcsManager {
    // Entities are just objects that contain an ID and some component masks
    // Entities are linked to multiple components, but they don't store the component data by themselves
    pub(crate) entities: EntitySet,

    // Archetypes are a subset of entities that all share the same component mask
    // We use an archetypal ECS because it is a bit more efficient when iterating through components, though it is slower when modifying entity component layouts
    pub(crate) archetypes: ArchetypeSet,

    // The unique storage set serves as a base where we can store empty versions of the vectors that are stored within the archetypes
    pub(crate) uniques: UniqueStoragesSet,

    // Was the ecs manager executed already?
    executed: bool,
}

impl Default for EcsManager {
    fn default() -> Self {
        // Create the default empty archetype
        let uniques: UniqueStoragesSet = Default::default();
        let empty = Archetype::new(Mask::zero(), &uniques);

        Self {
            entities: Default::default(),
            archetypes: MaskMap::from_iter(std::iter::once((Mask::zero(), empty))),
            uniques,
            executed: false,
        }
    }
}

impl Resource for EcsManager {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update(&mut self) {
        self.prepare();
    }
}

impl EcsManager {
    // Prepare the ecs for one frame of execution
    pub(crate) fn prepare(&mut self) {
        if !self.executed {
            for (_, a) in self.archetypes.iter_mut() {
                a.states().reset();
            }
        }
        self.executed = true;
    }

    // Modify an entity's component layout
    pub fn modify(
        &mut self,
        entity: Entity,
        function: impl FnOnce(&mut LinkModifier),
    ) -> Option<()> {
        // Keep a copy of the linkings before we do anything
        let mut copied = *self.entities.get(entity)?;

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(&mut linker);

        // Apply the changes
        linker.apply(&mut copied);
        *self.entities.get_mut(entity).unwrap() = copied;
        Some(())
    }

    // Get a single entity entry, if possible
    pub fn entry(&mut self, entity: Entity) -> Option<Entry> {
        Entry::new(self, entity)
    }

    // Insert an emtpy entity into the manager, and run a callback that will add components to it
    pub fn insert(&mut self, function: impl FnOnce(Entity, &mut LinkModifier)) -> Entity {
        // Add le entity
        let entity = self.entities.insert(EntityLinkings::default());

        // Create a link modifier, so we can insert/remove components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        function(entity, &mut linker);

        // Since we are inserting this entity, the linkings are always default
        let mut linkings = EntityLinkings::default();
        linker.apply(&mut linkings);
        *self.entities.get_mut(entity).unwrap() = linkings;

        entity
    }

    // Remove an entity from the world
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        // Remove the entity from it's current archetype first
        Archetype::remove(
            &mut self.archetypes,
            &mut self.entities,
            entity,
            Mask::zero(),
        );

        // Then remove it from the manager
        self.entities.remove(entity).unwrap();
        Some(())
    }

    // Get all the entities that are stored within the manager
    pub fn entities(&self) -> &EntitySet {
        &self.entities
    }

    // Get the archetypes
    pub fn archetypes(&self) -> &ArchetypeSet {
        &self.archetypes
    }

    /* #region Main thread queries */
    // Normal query without filter
    pub fn try_query<'a, Layout: QueryLayout<'a> + 'a>(
        &'a mut self,
    ) -> Option<impl Iterator<Item = Layout> + 'a> {
        Layout::validate().then(|| query(&self.archetypes))
    }

    // Create a query with a specific filter
    pub fn try_query_with<'a, Layout: QueryLayout<'a> + 'a, Filter: Evaluate>(
        &'a mut self,
        filter: Filter,
    ) -> Option<impl Iterator<Item = Layout> + 'a> {
        Layout::validate().then(|| filtered(&self.archetypes, filter))
    }

    // A view query that can only READ data, and never write to it
    // This will return None when it is unable to get a view query
    // TODO: Make use of Rust's type system to check for immutable borrows instead
    pub fn try_view<'a, Layout: QueryLayout<'a> + 'a>(
        &'a self,
    ) -> Option<impl Iterator<Item = Layout> + 'a> {
        let valid = Layout::combined().writing().empty() && Layout::validate();
        valid.then(|| query(&self.archetypes))
    }

    // View query with a specific filter
    pub fn try_view_with<'a, Layout: QueryLayout<'a> + 'a, Filter: Evaluate>(
        &'a self,
        filter: Filter,
    ) -> Option<impl Iterator<Item = Layout> + 'a> {
        let valid = Layout::combined().writing().empty() && Layout::validate();
        valid.then(|| filtered(&self.archetypes, filter))
    }
    /* #endregion */

    /* #region Parallel thread queries (with the help of rayon) */
    // TODO: Actually write this
    /* #endregion */
}
