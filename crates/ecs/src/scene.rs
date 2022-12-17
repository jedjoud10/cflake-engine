use ahash::{AHashMap, AHashSet};
use slotmap::SlotMap;
use std::iter::once;
use world::{post_user, user, System, World};

use crate::{
    archetype::remove_bundle, entity::Entity, Archetype,
    Bundle, EntityLinkings, EntryMut, EntryRef, Mask, MaskHashMap,
    QueryFilter, QueryLayoutMut, QueryLayoutRef, QueryMut, QueryRef,
    Wrap, Parent, Child,
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

    // Attach an entity to another entity, making a child-parent relation
    // Returns None if the entities don't exist, or if child is already attached
    pub fn attach(&mut self, child: Entity, parent: Entity) -> Option<()> {
        // Get the "Parent" component from the parent entity
        let mut parent_entry = self.entry_mut(parent)?;
        let parent_depth = if let Some(parent) = parent_entry.get_mut::<Parent>() {
            parent_node.children += 1;
            parent_node.depth
        } else {
            parent_entry.insert_bundle(Node {
                local_to_world: Default::default(),
                parent: None,
                children: 1,
                depth: 0,
            });
            
            0
        };

        // Update the child node, or insert it if it doesn't exist yet
        let mut child_entry = self.entry_mut(child)?;
        if let Some(child_node) = child_entry.get_mut::<Node>() {
            if let None = child_node.parent {
                child_node.parent = Some(parent);
                child_node.depth = parent_depth + 1;
            } else {
                return None
            }
        } else {
            child_entry.insert_bundle(Node {
                local_to_world: Default::default(),
                parent: Some(parent),
                children: 0,
                depth: parent_depth + 1,
            });
        }

        Some(())
    }

    // Detach an entity from it's parent
    // Returns None if the entities don't exist, or if the child isn't attached
    pub fn detach(&mut self, child: Entity) -> Option<()> {
        // Get the child node
        //let mut child_entry = self.entry_mut(child)?;
        //let mut child_node = child_entry.get_mut::<Node>()?;

        Some(())
    }

    // Move a child from one parent to another 
    // Equivalent to calling detach() then attach()
    pub fn orphan(&mut self, child: Entity, new_parent: Entity) {

    }
}

// Init event that will insert the ECS resource
fn init(world: &mut World) {
    world.insert(Scene::default());
}

// Reset the archetype states and update the hierarchy
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Clear all the archetype states that were set last frame
    for (_, archetype) in scene.archetypes_mut() {
        for (_, column) in archetype.state_table_mut().iter_mut() {
            column.clear();
        }
    }
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame (except the first frame)
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update).after(post_user);
}
