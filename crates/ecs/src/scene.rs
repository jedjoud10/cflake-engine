use ahash::AHashMap;
use itertools::Itertools;
use slotmap::SlotMap;
use std::iter::once;
use world::{post_user, user, System, World};

use crate::{
    contains, entity::Entity, mask, Archetype, Bundle, Child,
    Component, EntityLinkings, EntryMut, EntryRef, LocalPosition,
    LocalRotation, LocalScale, Mask, MaskHashMap, Parent, Position,
    QueryFilter, QueryLayoutMut, QueryLayoutRef, QueryMut, QueryRef,
    Rotation, Scale, UntypedVec, Wrap,
};

// Convenience type aliases
pub(crate) type EntitySet = SlotMap<Entity, EntityLinkings>;
pub(crate) type ArchetypeSet = MaskHashMap<Archetype>;
pub(crate) type RemovedComponents = MaskHashMap<Box<dyn UntypedVec>>;

// The scene is what will contain the multiple ECS entities and archetypes
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
}

impl Default for Scene {
    fn default() -> Self {
        let mut empty = Archetype::empty();
        empty.shrink_to_fit();
        Self {
            entities: Default::default(),
            archetypes: ArchetypeSet::from_iter(once((
                Mask::zero(),
                empty,
            ))),
            removed: Default::default(),
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
            .or_insert_with(|| Archetype::from_bundle::<B>());

        // Extend the archetype with the new bundles
        archetype.extend_from_iter::<B>(&mut self.entities, iter)
    }

    // Despawn an entity from the scene
    // Panics if the entity ID is invalid
    pub fn remove(&mut self, entity: Entity) {
        let linkings = *self.entities.get(entity).unwrap();
        let archetype =
            self.archetypes.get_mut(&linkings.mask).unwrap();
        archetype.remove_from_iter(
            &mut self.entities,
            [(entity, linkings)].into_iter(),
            &mut self.removed,
        );
    }

    // Despawn a batch of entities from an iterator
    // Panics if ANY entity ID is invalid
    pub fn remove_from_iter(
        &mut self,
        iter: impl IntoIterator<Item = Entity>,
    ) {
        // Convert the mask and group to vector of entities and linkings
        fn map(
            entities: &EntitySet,
            mask: Mask,
            group: impl IntoIterator<Item = Entity>,
        ) -> (Mask, Vec<(Entity, EntityLinkings)>) {
            let vec = group
                .into_iter()
                .map(|entity| {
                    let linkings =
                        entities.get(entity).cloned().unwrap();
                    (entity, linkings)
                })
                .collect::<Vec<_>>();
            (mask, vec)
        }

        // Group the entities based on their archetype
        let binding = iter.into_iter().group_by(|entity| {
            let linkings = self.entities.get(*entity).unwrap();
            linkings.mask()
        });

        // Fetch the entities that correspond to each archetype
        let iter = binding
            .into_iter()
            .map(|(mask, group)| map(&self.entities, mask, group))
            .into_iter()
            .collect::<Vec<_>>();

        // Batch remove the entities per archetype
        for (mask, group) in iter {
            let archetype = self.archetypes.get_mut(&mask).unwrap();

            // Remove the entities from the group
            archetype.remove_from_iter(
                &mut self.entities,
                group.into_iter(),
                &mut self.removed,
            );
        }
    }

    // Fetch all the removed components of a specific type immutably
    pub fn removed<T: Component + Default>(&self) -> &[T] {
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

    // Fetch all the removed components of a specific type mutably
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

    // Get a immutable reference to the active archetype set
    pub fn archetypes(&self) -> &ArchetypeSet {
        &self.archetypes
    }

    // Get a mutable reference to the active archetype set
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
    pub fn query_mut<'a, L: QueryLayoutMut>(
        &'a mut self,
    ) -> QueryMut<'a, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new(self)
    }

    // Create a new mutable query from this scene using a filter
    pub fn query_mut_with<'a, L: QueryLayoutMut>(
        &'a mut self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryMut<'a, '_, L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );
        QueryMut::new_with_filter(self, filter)
    }

    // Create a new immutable query from this scene (with no filter)
    pub fn query<'a, L: QueryLayoutRef>(
        &'a self,
    ) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new(self)
    }

    // Create a new immutable query from this scene using a filter
    pub fn query_with<'a, L: QueryLayoutRef>(
        &'a self,
        filter: Wrap<impl QueryFilter>,
    ) -> QueryRef<'a, '_, '_, L> {
        QueryRef::new_with_filter(self, filter)
    }

    // Find the a layout ref (if it's the only one that exists in the scene)
    pub fn find<'a, L: QueryLayoutRef>(&'a self) -> Option<L> {
        let mut iterator = self.query::<L>().into_iter().fuse();
        iterator.next().xor(iterator.next())
    }

    // Find the a layout mut (if it's the only one that exists in the scene)
    pub fn find_mut<'a, L: QueryLayoutMut>(
        &'a mut self,
    ) -> Option<L> {
        let mut iterator = self.query_mut::<L>().into_iter().fuse();
        iterator.next().xor(iterator.next())
    }

    // Attach an entity to another entity, making a child-parent relation
    // Returns None if the entities don't exist, or if child is already attached
    pub fn attach(
        &mut self,
        child: Entity,
        parent: Entity,
    ) -> Option<()> {
        // Get the "Parent" component from the parent entity
        let mut parent_entry = self.entry_mut(parent)?;
        if parent_entry.get_mut::<Parent>().is_none() {
            parent_entry.insert(Parent);
        }
        let parent_depth = parent_entry
            .get::<Child>()
            .map(|c| c.depth())
            .unwrap_or_default();

        // Update the child node, or insert it if it doesn't exist yet
        let mut child_entry = self.entry_mut(child)?;
        if let Some(child) = child_entry.get_mut::<Child>() {
            child.parent = parent;
            child.depth = parent_depth + 1;
        } else {
            child_entry.insert(Child {
                parent,
                depth: parent_depth + 1,
            });
        }

        Some(())
    }

    // Detach an entity from it's parent
    // Returns None if the entities don't exist, or if the child isn't attached
    pub fn detach(&mut self, child: Entity) -> Option<()> {
        let mut entry = self.entry_mut(child)?;
        assert!(entry.remove::<Child>());

        // Remove the "local" components that we added automatically
        entry.remove::<LocalPosition>();
        entry.remove::<LocalRotation>();
        entry.remove::<LocalScale>();

        Some(())
    }
}

// Init event that will insert the ECS resource
fn init(world: &mut World) {
    world.insert(Scene::default());
}

// Reset the archetypes
fn reset_states(world: &mut World) {
    // Clear all the archetype states that were set last frame
    let mut scene = world.get_mut::<Scene>().unwrap();
    for (_, archetype) in scene.archetypes_mut() {
        for (_, column) in archetype.table_mut().iter_mut() {
            column.states_mut().reset();
        }
    }
}

// Update the hierarchy
fn update_hierarchy(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Keeps track of the global transform of parents
    type Transform =
        (Option<Position>, Option<Rotation>, Option<Scale>);
    let mut transforms = AHashMap::<Entity, Transform>::new();

    // Fetch entities that are roots (ONLY parents)
    // TODO: Optimize this by only checking only the parents that have been modified, and updating their subtree ONLY if the parent transform got modified
    let filter = contains::<Parent>() & !contains::<Child>();
    for (entity, pos, rot, scl) in scene.query_with::<(
        &Entity,
        Option<&Position>,
        Option<&Rotation>,
        Option<&Scale>,
    )>(filter)
    {
        transforms.insert(
            *entity,
            (pos.cloned(), rot.cloned(), scl.cloned()),
        );
    }

    // Iterate recursively until all the entities finished updating their locations
    let mut recurse = true;
    while recurse {
        recurse = false;

        // Iterate through all the child entities and update their global transform based on the parent
        for (
            entity,
            child,
            local_pos,
            local_rot,
            local_scale,
            global_pos,
            global_rot,
            global_scl,
        ) in scene.query_mut::<(
            &Entity,
            &Child,
            Option<&LocalPosition>,
            Option<&LocalRotation>,
            Option<&LocalScale>,
            Option<&mut Position>,
            Option<&mut Rotation>,
            Option<&mut Scale>,
        )>() {
            if let Some(parent_transform) =
                transforms.get(&child.parent)
            {
                let (parent_position, parent_rotation, parent_scale) =
                    parent_transform;

                // Zip the required elements
                let pos =
                    global_pos.zip(local_pos).zip(*parent_position);
                let rot =
                    global_rot.zip(local_rot).zip(*parent_rotation);
                let scl =
                    global_scl.zip(local_scale).zip(*parent_scale);

                // Update the global position based on the parent position (and parent rotation)
                let global_pos =
                    if let Some(((global, local), parent)) = pos {
                        if let Some(parent_rotation) = parent_rotation
                        {
                            **global =
                                vek::Mat4::from(parent_rotation)
                                    .mul_point(**local + *parent);
                        } else {
                            **global = **local + *parent;
                        }

                        Some(*global)
                    } else {
                        None
                    };

                // Update the global rotation based on the parent rotation
                let global_rot =
                    if let Some(((global, local), parent)) = rot {
                        **global = **local * *parent;
                        Some(*global)
                    } else {
                        None
                    };

                // Update the global scale based on the parent scale
                let global_scl =
                    if let Some(((global, local), parent)) = scl {
                        **global = **local * *parent;
                        Some(*global)
                    } else {
                        None
                    };

                // Act as if the child we just updated is a parent itself
                transforms.insert(
                    *entity,
                    (global_pos, global_rot, global_scl),
                );
            } else {
                // We must repeat this for another pass it seems
                recurse = true;
            }
        }
    }
}

// The ECS system will manually insert the ECS resource and will clean it at the start of each frame
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(reset_states).before(user);
}

// This system will update the scene hierarchy with the proper local offsets and rotations
pub fn hierarchy(system: &mut System) {
    system.insert_update(update_hierarchy).after(post_user);
}
