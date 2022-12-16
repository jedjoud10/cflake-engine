use slotmap::SlotMap;
use std::iter::once;
use world::{post_user, user, System, World};

use crate::{
    archetype::remove_bundle, entity::Entity, Archetype,
    Bundle, EntityLinkings, EntryMut, EntryRef, Mask, MaskHashMap,
    QueryFilter, QueryLayoutMut, QueryLayoutRef, QueryMut, QueryRef,
    Wrap,
};

// Convenience type aliases
pub(crate) type EntitySet = SlotMap<Entity, EntityLinkings>;
pub(crate) type ArchetypeSet = MaskHashMap<Archetype>;

// The scene is what will contain the multiple ECS entities and archetypes
pub struct Scene {
    // Entities are just objects that contain an ID and some component masks
    // Entities are linked to multiple components, but they don't store the component data by themselves
    pub(crate) entities: EntitySet,

    // Archetypes are a subset of entities that all share the same component mask
    // We use an archetypal ECS because it is a bit more efficient when iterating through components, though it is slower when modifying entity component layouts
    pub(crate) archetypes: ArchetypeSet,
}

impl Default for Scene {
    fn default() -> Self {
        let mut empty = Archetype::empty();
        empty.shrink();
        Self {
            entities: Default::default(),
            archetypes: MaskHashMap::from_iter(once((
                Mask::zero(),
                empty,
            ))),
        }
    }
}

impl Scene {
    // Spawn an entity with specific components
    pub fn insert<B: Bundle>(&mut self, components: B) -> Entity {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );
        self.extend_from_iter(once(components))[0]
    }

    // Spawn a batch of entities with specific components from an iterator
    pub fn extend_from_iter<B: Bundle>(
        &mut self,
        iter: impl IntoIterator<Item = B>,
    ) -> &[Entity] {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Try to get the archetype, and create a default one if it does not exist
        let mask = B::reduce(|a, b| a | b);
        let archetype = self
            .archetypes
            .entry(mask)
            .or_insert_with(|| Archetype::from_table_accessor::<B>());
        let components = iter.into_iter().collect::<Vec<_>>();

        // Extend the archetype with the new bundles
        archetype
            .extend_from_slice::<B>(&mut self.entities, components)
    }

    // Remove an entity, and discard it's components
    pub fn remove(&mut self, entity: Entity) -> Option<()> {
        self.remove_from_iter(once(entity))
    }

    // Remove an entity, and fetch it's removed components as a new bundle
    pub fn remove_then<B: Bundle>(
        &mut self,
        entity: Entity,
    ) -> Option<B> {
        self.remove_from_iter_then::<B>(once(entity))
            .map(|mut vec| vec.pop().unwrap())
    }

    // Remove multiple entities, and discard their components
    pub fn remove_from_iter(
        &mut self,
        iter: impl IntoIterator<Item = Entity>,
    ) -> Option<()> {
        for entity in iter.into_iter() {
            let linkings = *self.entities.get(entity)?;
            let archetype =
                self.archetypes.get_mut(&linkings.mask).unwrap();
            archetype.remove(&mut self.entities, entity).unwrap();
        }

        Some(())
    }

    // Remove multiple entities, and fetch their removed components as new bundles
    pub fn remove_from_iter_then<B: Bundle>(
        &mut self,
        iter: impl IntoIterator<Item = Entity>,
    ) -> Option<Vec<B>> {
        iter.into_iter()
            .map(|entity| {
                // Move the entity from it's current archetype to the unit archetype
                remove_bundle::<B>(
                    &mut self.archetypes,
                    entity,
                    &mut self.entities,
                )
                .map(|bundle| {
                    self.entities.remove(entity).unwrap();
                    bundle
                })
            })
            .collect::<Option<Vec<B>>>()
    }

    // Check if an entity is stored within the scene
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains_key(entity)
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

    // Create a new mutable query from this scene (with no filter)
    pub fn query_mut<'a, L: for<'i> QueryLayoutMut<'i>>(
        &'a mut self,
    ) -> QueryMut<'a, '_, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new(self)
    }

    // Create a new mutable query from this scene using a filter
    pub fn query_mut_with<'a, L: for<'i> QueryLayoutMut<'i>>(
        &'a mut self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryMut<'a, '_, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new_with_filter(self, filter)
    }

    // Create a new immutable query from this scene (with no filter)
    pub fn query<'a, L: for<'i> QueryLayoutRef<'i>>(
        &'a self,
    ) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new(self)
    }

    // Create a new immutable query from this scene using a filter
    pub fn query_with<'a, L: for<'i> QueryLayoutRef<'i>>(
        &'a self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new_with_filter(self, filter)
    }

    // Find the a layout ref (if it's the only one that exists in the scene)
    pub fn find<'a, L: for<'i> QueryLayoutRef<'i>>(
        &'a self,
    ) -> Option<L> {
        let mut iterator =
            QueryRef::<L>::new(self).into_iter().fuse();
        iterator.next().xor(iterator.next())
    }

    // Find the a layout mut (if it's the only one that exists in the scene)
    pub fn find_mut<'a, L: for<'i> QueryLayoutMut<'i>>(
        &'a mut self,
    ) -> Option<L> {
        let mut iterator =
            QueryMut::<L>::new(self).into_iter().fuse();
        iterator.next().xor(iterator.next())
    }
}

// Late update event that will cleanup the ECS manager states
fn cleanup(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();

    // Clear all the archetype states that were set last frame
    for (_, archetype) in ecs.archetypes_mut() {
        for (_, column) in archetype.state_table_mut().iter_mut() {
            column.clear();
        }
    }
}

// Init event that will insert the ECS resource
fn init(world: &mut World) {
    world.insert(Scene::default());
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame (except the first frame)
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(cleanup).after(post_user);
}
