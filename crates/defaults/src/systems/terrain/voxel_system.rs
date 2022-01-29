use main::{
    core::{Context, WriteContext},
    ecs::{component::{ComponentQuery, ComponentID}, entity::EntityID}, terrain::{ChunkCoords, MAIN_CHUNK_SIZE, VoxelData, Voxel}, rendering::{advanced::{compute::ComputeShaderExecutionSettings, atomic::{AtomicGroup, AtomicGroupRead}}, basics::{uniforms::ShaderUniformsGroup, texture::{TextureAccessType, TextureReadBytes}, transfer::Transferable}, pipeline::{pipec, Pipeline}, object::PipelineTrackedTask},
};

use crate::globals::TerrainGenerationData;

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    // Create the compute shader execution settings and execute the compute shader
    let compute = terrain.base_compute;
    const AXIS: u16 = (MAIN_CHUNK_SIZE + 1) as u16 / 8 + 1;

    // Set the uniforms for the compute shader as well
    let mut group = ShaderUniformsGroup::new();
    
    // Chunk specific uniforms
    let chunk_coords = chunk.coords;
    group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
    group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
}


// The voxel systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read().unwrap();
    
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk in the terrain, we must create it's respective voxel data, if possible
        if terrain.generating.is_none() {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that we come across that needs it
            query.update_all_breakable(|components| {
                // We break out at the first chunk if we start generating it's voxel data
                let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                if chunk.voxel_data.is_none() {
                    // We must start generating the voxel data for this chunk
                    start_generation(&mut *terrain, &*pipeline, &mut *chunk, components.get_entity_id().unwrap());
                    None
                } else {
                    Some(())
                }
            })
        } else {
            // We must check if we have finished generating or not
            if pipec::has_task_executed(terrain.generating.as_ref().unwrap().main_id, &*pipeline).unwrap() {
                // We will now update the chunk data to store our new voxel data
                let id = terrain.generating.as_ref().unwrap().chunk_id;
                query.update(id, |components| {
                    // Get our chunk and set it's new data
                    let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                    finish_generation(&mut *terrain, &*pipeline, &mut *chunk);
                });
            }
        }
    }
}
// Create a voxel system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
