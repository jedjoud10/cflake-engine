use slotmap::SlotMap;
use time::Time;
use world::{Events, Init, Resource, Stage, Update, World};

use crate::{
    entity::Entity, Archetype, EntityLinkings, Entry, Evaluate, LinkError,
    Mask, MaskMap, MutEntry, OwnedBundle, QueryLayout, ComponentTable, query, query_filtered, ViewLayout, view, view_filtered,
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
            archetypes: MaskMap::from_iter(std::iter::once((Mask::zero(), Archetype::new_empty()))),
        }
    }
}

impl EcsManager {
    // Add an entity
    // Add a batch of entities
    // Remove an entity, and discard it's components
    // Remove an entity, and fetch it's removed components as a new bundle
    // Remove multiple entities, and discard their components
    // Remove multiple entities, and fetch their removed components as new bundles
    

    // Try to fetch a mutable entry for an entity
    /*
    pub fn mut_entry(&mut self, entity: Entity) -> Option<MutEntry> {
        MutEntry::new(self, entity)
    }

    // Try to fetch an immutable entry for an entity
    pub fn entry(&self, entity: Entity) -> Option<Entry> {
        Entry::new(self, entity)
    }

    // Insert an entity with the given component set as a tuple
    pub fn insert<T: OwnedComponentLayout>(&mut self, tuple: T) -> Result<Entity, LinkError> {
        self.insert_with(|_| tuple)
    }
    // Insert an entity with the given component set as a tuple using a callback
    pub fn insert_with<T: OwnedComponentLayout>(
        &mut self,
        callback: impl FnOnce(Entity) -> T,
    ) -> Result<Entity, LinkError> {
        let entity = self.entities.insert(EntityLinkings::default());

        // Create the modifier and insert the components
        let mut linker = LinkModifier::new(self, entity).unwrap();
        T::insert(callback(entity), &mut linker)?;

        // Create the linkings and apply the modifier
        let mut linkings = EntityLinkings::default();
        linker.apply(&mut linkings);
        *self.entities.get_mut(entity).unwrap() = linkings;

        Ok(entity)
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

    /*
    /* #region Main thread queries */
    // Normal query without filter
    pub fn query<'a, L: QueryLayout<'a> + 'a>(
        &'a mut self,
    ) -> Option<impl ExactSizeIterator<Item = L> + 'a> {
        L::validate().then(|| query(&mut self.archetypes))
    }

    // Create a query with a specific filter
    pub fn query_with<'a, L: QueryLayout<'a> + 'a>(
        &'a mut self,
        filter: impl Evaluate,
    ) -> Option<impl Iterator<Item = L> + 'a> {
        L::validate().then(|| query_filtered(&mut self.archetypes, filter))
    }

    // A view query that can only READ data, and never write to it
    pub fn view<'a, L: ViewLayout<'a> + 'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = L> + 'a {
        view(&self.archetypes)
    }

    // View query with a specific filter
    pub fn view_with<'a, L: ViewLayout<'a> + 'a>(
        &'a self,
        filter: impl Evaluate,
    ) -> impl Iterator<Item = L> + 'a {
        view_filtered(&self.archetypes, filter)
    }
    */
    */
    /* #endregion */
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
