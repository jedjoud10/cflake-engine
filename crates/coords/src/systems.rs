/*

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
 */

/*

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
 */

/*

// This system will update the scene hierarchy with the proper local offsets and rotations
pub fn hierarchy(system: &mut System) {
    system.insert_update(update_hierarchy).after(post_user);
}

 */