use ahash::AHashMap;
use itertools::Itertools;
use slotmap::SlotMap;
use std::iter::once;
//use world::{post_user, user, System, World};

use crate::{
    entity::Entity, mask, Archetype, Bundle, Component, EntityLinkings, EntryMut, EntryRef, Mask,
    MaskHashMap, PrefabBundle, QueryFilter, QueryLayoutMut, QueryLayoutRef, QueryMut, QueryRef,
    UntypedVec, Wrap,
};

// Convenience type aliases
pub(crate) type EntitySet = SlotMap<Entity, EntityLinkings>;
pub(crate) type ArchetypeSet = MaskHashMap<Archetype>;
pub(crate) type RemovedComponents = MaskHashMap<Box<dyn UntypedVec>>;

/// Identifier for prefabs
pub type PrefabId = &'static str;

/// The scene is what will contain the multiple ECS entities and archetypes
pub struct Scene {
    // Entities are just objects that contain an ID and some component masks
    // Entities are linked to multiple components, but they don't store the component data by themselves
    pub(crate) entities: EntitySet,

    // Archetypes are a subset of entities that all share the same component mask
    // We use an archetypal ECS because it is a bit more efficient when iterating through components,
    // though it is slower when modifying entity component layouts
    pub(crate) archetypes: ArchetypeSet,

    // These are removed components that we can iterate over
    // These components get added here whenever we destroy entities or unlink components from them
    // Stored as Box<Vec<T>> where T: Component
    pub(crate) removed: RemovedComponents,

    // These contain the boxed bundles that can be used for prefab generation
    pub(crate) prefabs: AHashMap<PrefabId, (Box<dyn PrefabBundle>, Mask)>,

    pub(crate) ticked: bool,
}

impl Default for Scene {
    fn default() -> Self {
        let mut empty = Archetype::empty();
        empty.shrink_to_fit();
        Self {
            entities: Default::default(),
            archetypes: ArchetypeSet::from_iter(once((Mask::zero(), empty))),
            removed: Default::default(),
            prefabs: AHashMap::default(),
            ticked: false,
        }
    }
}

impl Scene {
    /// Spawn an entity with specific components using the given [bundle](Bundle).
    pub fn insert<B: Bundle>(&mut self, components: B) -> Entity {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );
        self.extend_from_iter(once(components))[0]
    }

    /// Spawn a batch of entities with specific components from an iterator that creates [bundles](Bundle).
    pub fn extend_from_iter<B: Bundle>(&mut self, iter: impl IntoIterator<Item = B>) -> &[Entity] {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Try to get the archetype, and create a default one if it does not exist
        let mask = B::reduce(|a, b| a | b);
        let archetype = self
            .archetypes
            .entry(mask)
            .or_insert_with(|| Archetype::from_bundle::<B>());

        // Extend the archetype with the new bundles
        archetype.extend_from_iter::<B>(&mut self.entities, iter)
    }

    /// Despawn an entity from the scene.
    /// Panics if the entity ID is invalid.
    pub fn remove(&mut self, entity: Entity) {
        let linkings = *self.entities.get(entity).unwrap();
        let archetype = self.archetypes.get_mut(&linkings.mask).unwrap();
        archetype.remove_from_iter(
            &mut self.entities,
            [(entity, linkings)].into_iter(),
            &mut self.removed,
        );
    }

    /// Despawn a batch of entities from an iterator.
    /// Panics if ANY entity ID is invalid.
    pub fn remove_from_iter(&mut self, iter: impl IntoIterator<Item = Entity>) {
        // Sort the entities by their masks (we can use unstable since the ordering of the entities does not matter)
        let mut entities = iter
            .into_iter()
            .map(|e| (e, *self.entities.get(e).unwrap()))
            .collect::<Vec<_>>();
        entities.sort_unstable_by_key(|(_, l)| l.mask);

        // Group the entities based on their archetype
        let grouped = entities.iter().group_by(|(_, l)| l.mask);

        // Fetch the entities that correspond to each archetype
        let iter = grouped
            .into_iter()
            .map(|(key, group)| (key, group.into_iter().collect::<Vec<_>>()))
            .collect::<Vec<_>>();

        // Batch remove the entities per archetype
        for (mask, group) in iter {
            let archetype = self.archetypes.get_mut(&mask).unwrap();

            // If the group contains the same number of entities, just clear the archetype directly
            if group.len() == archetype.entities().len() {
                archetype.clear();
            } else {
                // Remove the entities from the archetype
                archetype.remove_from_iter(
                    &mut self.entities,
                    group.into_iter().cloned(),
                    &mut self.removed,
                );
            }
        }
    }

    /// Fetch all the removed components of a specific type immutably.
    pub fn removed<T: Component>(&self) -> &[T] {
        self.removed
            .get(&mask::<T>())
            .map(|untyped| {
                untyped
                    .as_any()
                    .downcast_ref::<Vec<T>>()
                    .unwrap()
                    .as_slice()
            })
            .unwrap_or(&[])
    }

    /// Fetch all the removed components of a specific type mutably.
    pub fn removed_mut<T: Component>(&mut self) -> &mut [T] {
        self.removed
            .get_mut(&mask::<T>())
            .map(|untyped| {
                untyped
                    .as_any_mut()
                    .downcast_mut::<Vec<T>>()
                    .unwrap()
                    .as_mut_slice()
            })
            .unwrap_or(&mut [])
    }

    /// Instantiate a prefab using it's prefab name and return a mutable entry.
    pub fn instantiate(&mut self, name: PrefabId) -> Option<EntryMut> {
        let (boxed, mask) = self.prefabs.get(name)?;
        let archetype = self.archetypes.get_mut(mask).unwrap();
        let entity = archetype.instantiate_prefab(&mut self.entities, boxed);
        self.entry_mut(entity)
    }

    /// Add a new bundle as a prefab so we can clone it multiple times.
    pub fn prefabify<B: Bundle + Clone>(&mut self, name: PrefabId, bundle: B) {
        // Try to get the archetype, and create a default one if it does not exist
        let mask = B::reduce(|a, b| a | b);
        self.archetypes
            .entry(mask)
            .or_insert_with(|| Archetype::from_bundle::<B>());

        let boxed: Box<dyn PrefabBundle> = Box::new(bundle);
        self.prefabs.insert(name, (boxed, mask));
    }

    /*
    /// Fetch an entry for a corresponding entity with the given name
    pub fn find_by_name_mut(&mut self, name: &str) -> Option<EntryMut> {
        todo!()
    }

    /// Fetch an entry for a corresponding entity with the given tag
    pub fn find_by_tag_mut(&mut self, tag: &str) -> Option<EntryMut> {
        todo!()
    }
    */

    /// Get the internally stored prefab hashmap.
    pub fn prefabs(&self) -> &AHashMap<PrefabId, (Box<dyn PrefabBundle>, Mask)> {
        &self.prefabs
    }

    /// Check if an entity is stored within the scene.
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains_key(entity)
    }

    /// Get the immutable entity [entry](EntryRef) for a specific entity.
    pub fn entry(&self, entity: Entity) -> Option<EntryRef> {
        EntryRef::new(self, entity)
    }

    /// Get the mutable entity [entry](EntryMut) for a specific entity.
    pub fn entry_mut(&mut self, entity: Entity) -> Option<EntryMut> {
        EntryMut::new(self, entity)
    }

    /// Get a immutable reference to the active [archetype set](ArchetypeSet).
    pub fn archetypes(&self) -> &ArchetypeSet {
        &self.archetypes
    }

    /// Get a mutable reference to the active [archetype set](ArchetypeSet).
    pub fn archetypes_mut(&mut self) -> &mut ArchetypeSet {
        &mut self.archetypes
    }

    /// Get an immutable reference to the [entity set](EntitySet).
    pub fn entities(&self) -> &EntitySet {
        &self.entities
    }

    /// Get a mutable reference to the [entity set](EntitySet).
    pub fn entities_mut(&mut self) -> &mut EntitySet {
        &mut self.entities
    }

    /// Create a new mutable [query](QueryLayoutMut) from this scene (with no filter).
    pub fn query_mut<'a, L: QueryLayoutMut>(&'a mut self) -> QueryMut<'a, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new(self)
    }

    /// Create a new mutable [query](QueryLayoutMut) from this scene using a [filter](QueryFilter).
    pub fn query_mut_with<'a, L: QueryLayoutMut>(
        &'a mut self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryMut<'a, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new_with_filter(self, filter, self.ticked)
    }

    /// Create a new immutable [query](QueryLayoutRef) from this scene (with no filter).
    pub fn query<'a, L: QueryLayoutRef>(&'a self) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new(self)
    }

    /// Create a new immutable [query](QueryLayoutRef) from this scene using a [filter](QueryFilter).
    pub fn query_with<'a, L: QueryLayoutRef>(
        &'a self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new_with_filter(self, filter, self.ticked)
    }

    /// Find the a layout ref (if it's the only one that exists in the scene).
    pub fn find<'a, L: QueryLayoutRef>(&'a self) -> Option<L> {
        let mut iterator = self.query::<L>().into_iter().fuse();
        iterator.next().xor(iterator.next())
    }

    /// Find the a layout mut (if it's the only one that exists in the scene).
    pub fn find_mut<'a, L: QueryLayoutMut>(&'a mut self) -> Option<L> {
        let mut iterator = self.query_mut::<L>().into_iter().fuse();
        iterator.next().xor(iterator.next())
    }
}

/*
// Init event that will insert the ECS resource
fn init(world: &mut World) {
    world.insert(Scene::default());
}

// At the end of each frame reset the delta states
fn reset_delta_frame_states_end(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    for (_, archetype) in scene.archetypes_mut() {
        for (_, column) in archetype.table_mut().iter_mut() {
            column.delta_frame_states_mut().reset();
        }
    }

    for (_, vec) in scene.removed.iter_mut() {
        vec.clear();
    }
}

// At the end of each tick reset the tick states
fn reset_delta_tick_states_end(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    for (_, archetype) in scene.archetypes_mut() {
        for (_, column) in archetype.table_mut().iter_mut() {
            column.delta_tick_states_mut().reset();
        }
    }
}

// Called at the start of every frame to set ticked to false
fn set_ticked_true(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.ticked = true;
}

// Called at the start of every tick to set ticked to true
fn set_ticked_false(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.ticked = false;
}

/// Only used for init
pub fn common(system: &mut System) {
    system.insert_init(init).before(user);
}

/// Executes shit at the start of every frame
pub fn pre_frame_or_tick(system: &mut System) {
    system
        .insert_tick(set_ticked_true)
        .before(user)
        .after(utils::time)
        .before(post_frame_or_tick);
    system
        .insert_update(set_ticked_false)
        .before(user)
        .after(utils::time)
        .before(post_frame_or_tick);
}

/// Executes shit at the end of each frame
pub fn post_frame_or_tick(system: &mut System) {
    system
        .insert_update(reset_delta_frame_states_end)
        .after(post_user)
        .after(utils::time)
        .after(pre_frame_or_tick);
    system
        .insert_tick(reset_delta_tick_states_end)
        .after(post_user)
        .after(utils::time)
        .after(pre_frame_or_tick);
}
*/