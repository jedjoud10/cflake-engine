use std::num::NonZeroU64;

use crate::{
    components::{Camera, Chunk, Transform},
    globals::{GlobalWorldData, Terrain},
};
use world::{input::Keys, terrain::ChunkCoords, World};

// The chunk systems' update loop
fn run(world: &mut World) {
    // Get the global terrain component
    // Get the camera position
    let (camera_position, camera_forward) = {
        let camera = world.globals.get::<GlobalWorldData>().unwrap().camera;
        let entry = world.ecs.entry(camera);
        if let Some(entry) = entry {
            let transform = entry.get::<Transform>().unwrap();
            (transform.position, transform.forward())
        } else {
            return;
        }
    };
    let terrain_ = world.globals.get_mut::<Terrain>();
    if world.input.toggled("update_terrain") || terrain_.is_none() {
        // No need to update the terrain
        return;
    }

    // Generate the chunks if needed and only if we are not currently generating
    let terrain = terrain_.unwrap();
    let manager = &mut terrain.manager;
    manager.must_update_octree = manager.octree.inner.must_update(camera_position);
    if manager.chunks_generating.is_empty() && manager.chunks_to_remove.is_empty() && terrain.scheduler.active_mesh_tasks_count() == 0 {
        let octree = &mut manager.octree;
        let size = octree.inner.size();
        if let Some((added, removed)) = octree.update(camera_position) {
            // We have moved, thus the chunks need to be regenerated
            // Remove chunks only if we already generated them
            for node in removed {
                let coords = ChunkCoords::new(&node);
                if let Some(id) = manager.chunks.remove(&coords) {
                    manager.chunks_to_remove.push(id);
                }
            }

            // Only add the chunks that are leaf nodes in the octree
            for node in added {
                if node.children().is_none() {
                    // This is a leaf node, so add it as an entity
                    let coords = ChunkCoords::new(&node);

                    // Le new ECS is very cool
                    let chunk = world.ecs.insert(|_, linker| {
                        // Link the nessecary components

                        // Transform
                        linker
                            .insert(Transform {
                                position: coords.position.as_(),
                                scale: vek::Vec3::one() * ((coords.size / size) as f32),
                                ..Default::default()
                            })
                            .unwrap();

                        // Chunk
                        linker.insert(Chunk {
                            coords,
                            voxel_data_id: None,
                            persistent: None,
                        }).unwrap();
                    });

                    // Also calculate the chunk's priority, so we know when to generate it
                    let priority = Chunk::calculate_priority(coords, camera_position, camera_forward);
                    manager.priority_list.push((chunk, priority));
                    manager.chunks.insert(coords, chunk);
                    manager.chunks_generating.insert(coords);
                }
            }
            manager.update_priorities();
        }
    } else {
        // Mass deletion when we have no more chunks
        if manager.chunks_generating.is_empty() && terrain.scheduler.active_mesh_tasks_count() == 0 {
            let chunks_to_remove = std::mem::take(&mut manager.chunks_to_remove);
            for entity in chunks_to_remove {
                // Simply remove the chunk from the world
                world.ecs.remove(entity);
            }
        }
    }
}

// Create a chunk system
pub fn system(world: &mut World) {
    world.events.insert(run);
    world.input.bind_toggle(Keys::Y, "update_terrain");
}
