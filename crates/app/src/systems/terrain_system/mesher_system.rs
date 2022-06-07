use crate::{
    components::{Chunk, Renderer, Transform},
    resources::ChunkGenerationState,
};
use rapier3d::prelude::{ColliderMaterial, RigidBodyType};
use world::{
    rendering::{
        basics::{material::Material, mesh::Mesh},
        pipeline::Handle,
    },
    terrain::mesher::{Mesher, MesherSettings},
    World,
};

// The mesher systems' update loop
fn run(world: &mut World) {
    let terrain = world.resources.get_mut::<crate::resources::Terrain>();
    if let Some(mut terrain) = terrain {
        // We can only create the mesh of a single chunk per frame
        if let ChunkGenerationState::EndVoxelDataGeneration(entity, true, Some(id)) = terrain.manager.current_chunk_state {
            // Get the chunk component from the specific chunk
            let mut entry = world.ecs.entry(entity).unwrap();
            let coords = entry.get_mut::<Chunk>().unwrap().coords;
            // Either way, we're going to be updating/generating the mesh so might as well make the mesher now
            let mesher = Mesher::new(
                coords,
                MesherSettings {
                    interpolation: true,
                    skirts: true,
                },
            );
            terrain.scheduler.execute(mesher, &terrain.generator.buffer, id);
            // We have created voxel data for this chunk, and it is valid (it contains a surface)
            terrain.manager.chunks_generating.remove(&coords);
            // Switch states
            terrain.manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
        } else if let ChunkGenerationState::EndVoxelDataGeneration(entity, false, _) = terrain.manager.current_chunk_state {
            // Remove the chunk's renderer if it had one
            if world.ecs.entry(entity).unwrap().get::<Renderer>().is_ok() {
                // Modify for removal ofc
                world.ecs.modify(entity, |entity, modifier| {
                    modifier.remove::<Renderer>().unwrap();
                });
            }

            // The chunk ID is the same, but we do not have a surface
            // We still gotta update the current chunk state though
            terrain.manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
        }

        // Get the meshes that were generated in other threads
        for generated in terrain.scheduler.get_results() {
            // Unlock
            let id = generated.id;
            let shared = terrain.generator.buffer.get(id).unwrap();
            let coords = generated.coords;

            // Build the mesh from the two builders
            let mesh = {
                let base = generated.base.build();
                let skirts = generated.skirts.build();
                let combined = Mesh::combine(base, skirts);
                world.pipeline.insert(combined)
            };

            // Get the chunk entity key
            if let Some(&entity) = terrain.manager.chunks.get(&coords) {
                // Get the entity
                let mut entry = world.ecs.entry(entity).unwrap();
                if entry.get::<Renderer>().is_err() {
                    // Generate the new component and link it

                    // Modify the entity
                    world
                        .ecs
                        .modify(entity, |_, modifier| {
                            // Create a renderer with the new mesh
                            modifier
                                .insert(Renderer {
                                    mesh: mesh.clone(),
                                    material: terrain.manager.material.clone(),
                                    ..Default::default()
                                })
                                .unwrap();
                        })
                        .unwrap();

                    // Update the chunk's voxel data,
                    let mut entry = world.ecs.entry(entity).unwrap();
                    entry.get_mut::<Chunk>().unwrap().voxel_data_id = Some(id);
                } else {
                    // Simply update the renderer
                    let renderer = entry.get_mut::<Renderer>().unwrap();
                    renderer.mesh = mesh;
                }

                // The chunk finished generation
                shared.set_used(false);
            }
        }
    }
}

// Create a mesher system
pub fn system(world: &mut World) {
    world.events.insert(run);
}