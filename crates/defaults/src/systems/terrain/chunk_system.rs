use core::global::callbacks::CallbackType;

use super::ChunkSystem;
use ecs::SystemData;
use math::octrees::OctreeNode;
use others::callbacks::MutCallback;
use terrain::{ChunkCoords, ChunkState};
ecs::impl_systemdata!(ChunkSystem);

// Create the chunk entity and add it to the world
fn create_chunk_entity(data: &mut SystemData<ChunkSystem>, coords: ChunkCoords, octree_size: u64) {
    // Create the entity
    let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
    let entity = ecs::Entity::new(name.as_str());

    // Create the chunk component
    let chunk = terrain::Chunk::new(coords);
    // Link the components
    let mut linkings = ecs::ComponentLinkingGroup::new();
    linkings.link::<terrain::Chunk>(chunk).unwrap();

    // Transform
    linkings
        .link::<crate::components::Transform>(
            crate::components::Transform::default()
                .with_position(coords.position.into())
                .with_scale(veclib::Vector3::new(
                    (coords.size / octree_size) as f32,
                    (coords.size / octree_size) as f32,
                    (coords.size / octree_size) as f32,
                )),
        )
        .unwrap();
    // Init the chunk state
    data.chunk_states.insert(coords, ChunkState::AwaitingCreation);
    // Add the entity
    let result = core::global::ecs::entity_add(entity, linkings);
    core::global::batch::batch_add(0, result);    
}

fn system_prefire(data: &mut SystemData<ChunkSystem>) {
    // We must add all the chunks that the octree tells us to add
    // First of all, we get the camera data
    let camera = core::global::ecs::entity(core::global::main::world_data().main_camera_entity_id).unwrap();
    let camera_transform = core::global::ecs::component::<crate::components::Transform>(&camera).unwrap();
    let camera_pos = camera_transform.position;

    // We are only allowed to update the octree in 2 conditions
    // 1. We do not have any current chunks, so we must initialize the octree
    // 2. All of the generated chunks have a chunk state of Valid, meaning that they have generated their Voxel Data and TModel successfully 
    let validity_test = data.chunk_states.iter().all(|(_, x)| *x == ChunkState::Valid );
    let valid = data.chunk_states.is_empty() || (validity_test);
    if valid {
        // Update the octree
        let octree = &mut data.octree;
        let octree_size = octree.internal_octree.size;
        if let Option::Some((mut added, removed, total)) = octree.generate_incremental_octree(&camera_pos, terrain::DEFAULT_LOD_FACTOR) {
            println!("Octree update");
            // Do a bit of filtering on the added nodes
            added.retain(|node| node.children_indices.is_none() && math::Intersection::csgtree_aabb(&data.csgtree, &node.get_aabb()));
            // Add the chunks into the world
            for x in added {
                let octree_node: OctreeNode = x;
                let coords = ChunkCoords::new(&octree_node);
                // Add the entity
                create_chunk_entity(data, coords, octree_size);
            }

            // Buffer the chunks that must be removed
            for octree_node in removed {
                let coords = ChunkCoords::new(&octree_node);
                // Get the entity ID, then we can remove it
                if let Option::Some(entity_id) = data.chunks.remove(&coords) {
                    // Set the state first
                    // Now we can actually remove the entity
                    let result = core::global::ecs::entity_remove(entity_id);
                    data.chunks_to_delete.insert(entity_id);
                    let x = data.chunk_states.get_mut(&coords).unwrap();
                    *x = ChunkState::AwaitingDeletion;
                    core::global::batch::batch_add(1, result);
                }
            }
        }
    }
    
    // If we are done generating the chunks, we can safely remove the old chunks
    core::global::batch::send_batch(1, false);
    if data.chunk_states.iter().all(|(_, x)| { *x == ChunkState::Valid }) && !data.chunks_to_delete.is_empty() {
        data.chunks_to_delete.clear();
    }  
    
    // We must send the "Chunk Creation" batch and the "Chunk Deletion" batch to the main thread
    core::global::batch::send_batch(0, false);
}

// We will loop through every chunk and update our internal state about them
fn entity_update(data: &mut SystemData<ChunkSystem>, entity: &ecs::Entity) {
    let chunk = core::global::ecs::component::<terrain::Chunk>(entity).unwrap();
    // Update the state
    if let Option::Some(internal_chunk_state) = data.chunk_states.get_mut(&chunk.coords) {
        *internal_chunk_state = chunk.state.clone();
    }
    // Check if the chunk has a renderer, because if it does then it's state becomes ValidRenderer
    core::global::ecs::entity_mut(entity.entity_id, CallbackType::LocalEntityMut(MutCallback::new(|entity: &mut ecs::Entity| {   
        let valid_renderer = match core::global::ecs::component::<crate::components::Renderer>(entity) {
            Ok(renderer) => renderer.internal_renderer.index.is_some(),
            Err(e) => { /* We simply do not have a renderer yet */ false },
        };
        let chunk = core::global::ecs::component_mut::<terrain::Chunk>(entity).unwrap();
        // Update the state if needed
        if valid_renderer && chunk.state == ChunkState::ValidModelData {
            chunk.state = ChunkState::Valid;
        }
    })).create());
}

// When we have added a Chunk Entity. This saves us from creating a callback actually
fn entity_added(data: &mut SystemData<ChunkSystem>, entity: &ecs::Entity) {
    // Add the chunk to our total chunks
    let mut data = data.clone();
    core::global::ecs::entity_mut(entity.entity_id, CallbackType::LocalEntityMut(MutCallback::new(move |entity: &mut ecs::Entity| {
        let entity_id = entity.entity_id;
        let chunk = core::global::ecs::component_mut::<terrain::Chunk>(entity).unwrap();
        let o = data.chunks.insert(chunk.coords, entity_id);
        if let Option::Some(_) = o { panic!() }
        *data.chunk_states.get_mut(&chunk.coords).unwrap() = ChunkState::ValidEntity;
        chunk.state = ChunkState::ValidEntity;
    })).create());
}

// We have removed a Chunk, we must remove it's corresponding data from our internal states as well
fn entity_removed(data: &mut SystemData<ChunkSystem>, entity: &ecs::Entity) {
    // Remove this chunk from our total chunks
    let chunk = core::global::ecs::component::<terrain::Chunk>(entity).unwrap();
    data.chunk_states.remove(&chunk.coords);
}

// Create the Chunk Manager system
// This system will add / remove chunks from the world and nothing else
pub fn system(depth: u8, csgtree: math::csg::CSGTree) {
    // Check if a an already existing node could be subdivided even more
    fn can_node_subdivide_twin(node: &math::octrees::OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
        let c: veclib::Vector3<f32> = node.get_center().into();
        let max = node.depth == 1;
        let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
        node.children_indices.is_none() && node.depth < max_depth && result
    }
    // Create a new octree
    let internal_octree = math::octrees::Octree::new(depth, (terrain::MAIN_CHUNK_SIZE) as u64);
    let octree = math::octrees::AdvancedOctree::new(internal_octree, can_node_subdivide_twin);

    // Create the system data from the given terrain settings
    let chunk_system_data = ChunkSystem {
        octree,
        csgtree,
        ..ChunkSystem::default()
    };
    core::global::ecs::add_system(chunk_system_data, || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<terrain::Chunk>();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        system.event(ecs::SystemEventType::EntityAdded(entity_added));
        system.event(ecs::SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
