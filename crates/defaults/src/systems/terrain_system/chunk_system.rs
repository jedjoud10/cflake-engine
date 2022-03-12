use crate::{
    components::{Camera, Transform},
    globals::{self, ChunksManager},
};
use world::{
    ecs::{
        self,
        component::{ComponentQueryParameters, ComponentQuerySet},
        entity::EntityKey,
        ECSManager,
    },
    input::Keys,
    terrain::ChunkCoords,
    World,
};

// Add a single chunk to the world
fn add_chunk(ecs: &mut ECSManager<World>, camera_position: veclib::Vector3<f32>, camera_forward: veclib::Vector3<f32>, octree_size: u64, coords: ChunkCoords) -> (EntityKey, f32) {
    // Create the chunk entity
    let _entity = ecs::entity::Entity::default();
    let mut group = ecs::entity::ComponentLinkingGroup::default();

    // Link the nessecary components
    // Transform
    let position = veclib::Vector3::<f32>::from(coords.position);
    let scale = veclib::Vector3::ONE * ((coords.size / octree_size) as f32);
    let transform = Transform {
        position,
        scale,
        ..Default::default()
    };
    group.link::<Transform>(transform).unwrap();

    // Calculate the chunk's priory and create it
    let chunk = crate::components::Chunk { coords };
    let priority = crate::components::Chunk::calculate_priority(coords, camera_position, camera_forward);
    group.link::<crate::components::Chunk>(chunk).unwrap();

    // Add the entity to the world
    let id = ecs.add(group).unwrap();
    (id, priority)
}
// Remove a single chunk
fn remove_chunk(ecs: &mut ECSManager<World>, id: EntityKey) {
    // Make sure that the chunk entity even exists
    if ecs.entities.get(id).is_ok() {
        // Remove the chunk entity at that specific EntityID
        ecs.remove(id).unwrap();
    }
}

// The chunk systems' update loop
fn run(world: &mut World, data: ComponentQuerySet) {
    // Get the global terrain component
    // Get the camera position
    let (camera_pos, camera_dir) = {
        let camkey = world.globals.get::<globals::GlobalWorldData>().unwrap().main_camera;
        let camquery = data.get(0).unwrap();
        let transform = camquery.all.get(&camkey).unwrap().get::<Transform>().unwrap();
        (transform.position, transform.forward())
    };
    let terrain_ = world.globals.get_mut::<globals::Terrain>();
    if world.input.map_toggled("update_terrain") || terrain_.is_err() {
        // No need to update the terrain
        return;
    }
    let terrain = terrain_.unwrap();
    // Generate the chunks if needed and only if we are not currently generating
    let handler = &mut terrain.chunks_manager;
    update_terrain(handler, camera_pos, &mut world.ecs, camera_dir);
}

// Update the terrain
fn update_terrain(handler: &mut ChunksManager, camera_position: veclib::Vector3<f32>, ecs: &mut ECSManager<World>, camera_forward: veclib::Vector3<f32>) {
    if handler.chunks_generating.is_empty() && handler.chunks_to_remove.is_empty() {
        let octree = &mut handler.octree;
        if let Some((added, removed)) = octree.update(camera_position) {
            // We have moved, thus the chunks need to be regenerated
            // Remove chunks only if we already generated them
            for node in removed {
                let coords = ChunkCoords::new(&node);
                if let Some(id) = handler.chunks.remove(&coords) {
                    handler.chunks_to_remove.push(id);
                }
            }

            // Only add the chunks that are leaf nodes in the octree
            for node in added {
                if node.children().is_none() {
                    // This is a leaf node
                    let coords = ChunkCoords::new(&node);
                    let (id, priority) = add_chunk(ecs, camera_position, camera_forward, octree.inner.size(), coords);
                    handler.priority_list.push((id, priority));
                    handler.chunks.insert(coords, id);
                    handler.chunks_generating.insert(coords);
                }
            }
            handler.update_priorities();
        }
    } else {
        // Mass deletion when we have no more chunks
        if handler.chunks_generating.is_empty() {
            let chunks_to_remove = std::mem::take(&mut handler.chunks_to_remove);
            for id in chunks_to_remove {
                remove_chunk(ecs, id);
            }
        }
    }
}
// Create a chunk system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .query(ComponentQueryParameters::default().link::<Camera>().link::<Transform>())
        .event(run)
        .build();
    world.input.bind_key_toggle(Keys::Y, "update_terrain");
}
