use std::time::Instant;

use crate::{components::Chunk, globals::ChunkGenerationState};
use world::{
    ecs::{
        entity::{ComponentLinkingGroup, ComponentUnlinkGroup},
        event::EventKey,
    },
    rendering::{
        basics::{material::Material, mesh::Mesh, renderer::RendererFlags},
        object::ObjectID,
        pipeline::pipec,
    },
    terrain::{
        mesher::{Mesher, MesherSettings},
        StoredVoxelData,
    },
    World,
};

// A post generation event that will be called after the generation of a specific chunk
fn chunk_post_gen(_world: &mut World, _chunk: &Chunk, _data: &StoredVoxelData) {}

// The mesher systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    let terrain = world.globals.get_mut::<crate::globals::Terrain>();
    if Instant::now().saturating_duration_since(world.time.current.begin_instant).as_millis() > 2 {
        return;
    }
    if let Ok(mut terrain) = terrain {
        // For each chunk that has a valid voxel data, we must create it's mesh
        for (&id, components) in query.iter_mut() {
            if terrain.chunks_manager.current_chunk_state == ChunkGenerationState::EndVoxelDataGeneration(id, true) {
                // We have created voxel data for this chunk, and it is valid (it contains a surface)
                let mut chunk = components.get_mut::<crate::components::Chunk>().unwrap();
                let voxel_data = &terrain.voxel_generator.stored_chunk_voxel_data;
                let mesher = Mesher::new(
                    chunk.coords,
                    voxel_data,
                    MesherSettings {
                        interpolation: true,
                        skirts: true,
                    },
                );
                // Create a linking group that add the renderer (only once)
                let chunk_entity = world.ecs.entities.get(id).unwrap();
                if !chunk_entity.is_component_linked::<crate::components::Renderer>() {
                    // Get the pipeline without angering the borrow checker
                    let pipeline = world.pipeline.read();
                    let material = terrain.chunks_manager.material;

                    // Construct the mesh and add it to the chunk entity
                    let mesh = mesher.build();
                    let mesh_id = pipec::construct(&pipeline, mesh).unwrap();
                    let group = create_chunk_renderer_linking_group(mesh_id, material);
                    // Link the group
                    world.ecs.link(id, group).unwrap();
                } else {
                    // The renderer is already linked, we just need to update the mesh
                    let pipeline = world.pipeline.read();
                    // Valid renderer
                    let mesh = mesher.build();
                    let mesh_id = pipec::construct(&pipeline, mesh).unwrap();
                    // We don't deconstruct the renderer since the terrain mesh update system will take care of that
                    chunk.updated_mesh_id = Some(mesh_id);
                }
                terrain.chunks_manager.chunks_generating.remove(&chunk.coords);
                // Switch states
                terrain.chunks_manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
                let voxel_data = &terrain.voxel_generator.stored_chunk_voxel_data.clone();
                chunk_post_gen(world, &chunk, voxel_data);
                return;
            } else if terrain.chunks_manager.current_chunk_state == ChunkGenerationState::EndVoxelDataGeneration(id, false) {
                let chunk = components.get_mut::<crate::components::Chunk>().unwrap();
                // Remove the chunk's renderer if it had one
                if world.ecs.entities.get(id).unwrap().is_component_linked::<crate::components::Renderer>() {
                    let mut unlink_group = ComponentUnlinkGroup::default();
                    unlink_group.unlink::<crate::components::Renderer>().unwrap();
                    world.ecs.unlink_components(id, unlink_group).unwrap();
                }

                // The chunk ID is the same, but we do not have a surface
                // We still gotta update the current chunk state though
                terrain.chunks_manager.current_chunk_state = ChunkGenerationState::RequiresVoxelData;
                let voxel_data = &terrain.voxel_generator.stored_chunk_voxel_data.clone();
                chunk_post_gen(world, &chunk, voxel_data);
                return;
            } else {
                // Skip since this is not the proper chunk
            }
        }
    }
}

// Create a new linking group that contains a renderer with a specific mesh
fn create_chunk_renderer_linking_group(mesh_id: ObjectID<Mesh>, material: ObjectID<Material>) -> ComponentLinkingGroup {
    // First time we link the renderer
    let mut group = ComponentLinkingGroup::default();
    group
        .link(crate::components::Renderer::new(RendererFlags::DEFAULT).with_mesh(mesh_id).with_material(material))
        .unwrap();
    group
}
// Create a mesher system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
