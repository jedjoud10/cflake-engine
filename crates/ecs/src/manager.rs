use slotmap::SlotMap;
use time::Time;
use world::{Events, Init, Resource, Stage, Update, World};

use crate::{
    entity::Entity, filtered, query, Archetype, EntityLinkings, Entry, Evaluate, LinkModifier,
    Mask, MaskMap, QueryLayout, StorageVec,
};

// Type aliases because I have gone insane
pub type EntitySet = SlotMap<Entity, EntityLinkings>;
pub type ArchetypeSet = MaskMap<Archetype>;
pub(crate) type UniqueStoragesSet = MaskMap<Box<dyn StorageVec>>;

// TODO: Find a better name for this bozo
#[derive(Resource)]
pub struct EcsManager {
    // Entities are just objects that contain an ID and some component masks
    // Entities are linked to multiple components, but they don't store the component data by themselves
    pub(crate) entities: EntitySet,

    // Archetypes are a subset of entities that all share the same component mask
    // We use an archetypal ECS because it is a bit more efficient when iterating through components, though it is slower when modifying entity component layouts
    pub(crate) archetypes: ArchetypeSet,

    // The unique storage set serves as a base where we can store empty versions of the vectors that are stored within the archetypes
    pub(crate) uniques: UniqueStoragesSet,
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
        }
    }
}

impl EcsManager {
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
    pub fn try_entry(&mut self, entity: Entity) -> Option<Entry> {
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
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame (except the first frame)
pub fn system(events: &mut Events) {
    // Update event that will cleanup the ECS manager states
    fn cleanup(world: &mut World) {
        let (ecs, time) = world.get_mut::<(&mut EcsManager, &Time)>().unwrap();

        // Ignore the first frame
        if time.frame_count() == 0 {
            return;
        }

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
    events.registry::<Init>().insert_with(
        init,
        Stage::new("ecs insert").before("user begin")
            .set_name("ecs insert")
            .set_before("main")
            .build()
            .unwrap(),
    );
    events.registry::<Update>().insert_with(
        cleanup,
        Stage::builder()
            .set_name("ecs cleanup")
            .set_before("main")
            .build()
            .unwrap(),
    );
}
