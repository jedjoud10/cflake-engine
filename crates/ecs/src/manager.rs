use itertools::Itertools;
use slotmap::SlotMap;
use time::Time;
use world::{Events, Init, Resource, Stage, Update, World};

use crate::{
    entity::Entity, Archetype,
    EntityLinkings, EntryRef, Evaluate, Mask, MaskMap, EntryMut, OwnedBundle,
    archetype::remove_bundle_unchecked, Bundle, MutQueryLayout, RefQueryLayout,
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
    pub fn insert<B: Bundle>(&mut self, components: B) -> Entity {
        assert!(B::is_valid());
        self.insert_from_iter(std::iter::once(components))[0]
    }

    // Spawn a batch of entities with specific components
    pub fn insert_from_iter<B: Bundle>(&mut self, iter: impl IntoIterator<Item = B>) -> Vec<Entity> {
        assert!(B::is_valid());

        // Try to get the archetype, and create a default one if it does not exist
        let mask = B::combined();
        let archetype = self.archetypes.entry(mask).or_insert_with(|| Archetype::from_table_accessor::<B>());
        let components = iter.into_iter().collect::<Vec<_>>();

        // Extend the archetype with the new bundles
        archetype.extend_from_slice::<B>(&mut self.entities, components)
    }

    // Remove an entity, and discard it's components
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        self.remove_from_iter(std::iter::once(entity))
    }

    // Remove an entity, and fetch it's removed components as a new bundle
    pub fn remove_then<B: Bundle>(&mut self, entity: Entity) -> Option<B> {
        assert!(B::is_valid());
        self.remove_from_iter_then::<B>(std::iter::once(entity)).map(|mut vec| vec.pop().unwrap())
    }

    // Remove multiple entities, and discard their components
    pub fn remove_from_iter(&mut self, iter: impl IntoIterator<Item = Entity>) -> Option<()> {
        for entity in iter.into_iter() {
            let linkings = *self.entities.get(entity)?;
            let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
            archetype.remove(&mut self.entities, entity).unwrap();
        }

        Some(())
    }

    // Remove multiple entities, and fetch their removed components as new bundles
    pub fn remove_from_iter_then<B: Bundle>(&mut self, iter: impl IntoIterator<Item = Entity>) -> Option<Vec<B>> {
        assert!(B::is_valid());

        iter.into_iter().map(|entity| {
            // Move the entity from it's current archetype to the unit archetype
            remove_bundle_unchecked::<B>(&mut self.archetypes, entity, &mut self.entities).map(|bundle| {
                self.entities.remove(entity).unwrap();
                bundle
            })
        }).collect::<Option<Vec<B>>>()
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
    pub fn query<'a, L: MutQueryLayout<'a>>(&mut self) -> Option<impl Iterator<Item = L> + 'a> {
        crate::query_mut(&mut self.archetypes)
    }  

    // Create a new mutable query iterator with a filter
    pub fn query_with<'a, L: MutQueryLayout<'a>>(&mut self, filter: impl Evaluate) -> Option<impl Iterator<Item = L> + 'a> {
        crate::query_mut_filter(&mut self.archetypes, filter)
    }
    
    // Create a new immutable query iterator
    pub fn view<'a, L: RefQueryLayout<'a>>(&self) -> Option<impl Iterator<Item = L> + 'a> {
        crate::query_ref(&self.archetypes)
    }

    // Create a new immutable query iterator with a filter
    pub fn view_with<'a, L: RefQueryLayout<'a>>(&self, filter: impl Evaluate) -> Option<impl Iterator<Item = L> + 'a> {
        crate::query_ref_filter(&self.archetypes, filter)
    }
  
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame (except the first frame)
pub fn system(events: &mut Events) {
    // Late update event that will cleanup the ECS manager states
    fn cleanup(world: &mut World) {
        let (ecs, _time) = world.get_mut::<(&mut EcsManager, &Time)>().unwrap();

        // Clear all the archetype states that were set last frame
        for (_, archetype) in ecs.archetypes() {
            let cloned = archetype.states();
            let mut states = cloned.borrow_mut();
            for state in states.iter_mut() {
                state.update(|added, removed, mutated| {
                    *added = Mask::zero();
                    *mutated = Mask::zero();
                    *removed = Mask::zero();
                });
            }
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
}
