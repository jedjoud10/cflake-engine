use main::{
    core::{Context, WriteContext},
    ecs::{component::{ComponentQuery, ComponentID}, entity::EntityID}, terrain::{ChunkCoords, MAIN_CHUNK_SIZE, VoxelData, Voxel}, rendering::{advanced::{compute::ComputeShaderExecutionSettings, atomic::{AtomicGroup, AtomicGroupRead}}, basics::{uniforms::ShaderUniformsGroup, texture::{TextureAccessType, TextureReadBytes}, transfer::Transferable}, pipeline::{pipec, Pipeline}, object::PipelineTrackedTask},
};

use crate::globals::TerrainGenerationData;

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    // Create the compute shader execution settings and execute the compute shader
    let compute = terrain.compute_shader;
    const AXIS: u16 = (MAIN_CHUNK_SIZE + 1) as u16 / 8 + 1;  
    // Set the uniforms for the compute shader as well
    let mut group = ShaderUniformsGroup::new();
    group.set_image("density_image", terrain.density_texture, TextureAccessType::WRITE);
    group.set_image("material_image", terrain.material_texture, TextureAccessType::WRITE);

    // Chunk specific uniforms
    let chunk_coords = chunk.coords;
    group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 1) as i32);
    group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
    group.set_i32("node_size", chunk_coords.size as i32);
    group.set_i32("depth", chunk_coords.depth as i32);

    // Create this for the next step
    let read_densities = TextureReadBytes::default();
    let read_materials = TextureReadBytes::default();
    let read_counters = AtomicGroupRead::default();
    let read_densities_transfer = read_densities.transfer();
    let read_materials_transfer = read_materials.transfer();
    let read_counters_transfer = read_counters.transfer();
    // Set the atomic counter
    group.set_atomic_group("_", terrain.counters, 2);

    // Now we can execute the compute shader and the read bytes command
    let execution_settings = ComputeShaderExecutionSettings::new(compute, (AXIS, AXIS, AXIS)).set_uniforms(group);
    let execution = pipec::tracked_task(PipelineTrackedTask::RunComputeShader(compute, execution_settings), None, pipeline);
    
    let read_densities_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.density_texture, read_densities_transfer), Some(execution), pipeline);
    let read_materials_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.material_texture, read_materials_transfer), Some(execution), pipeline);
    let read_counters_tracked_id = pipec::tracked_task(PipelineTrackedTask::AtomicGroupRead(terrain.counters, read_counters_transfer), Some(execution), pipeline);

    // Combine the tasks to make a finalizer one
    let main = pipec::tracked_finalizer(vec![execution, read_densities_tracked_id, read_materials_tracked_id, read_counters_tracked_id], pipeline).unwrap();
    terrain.generating = Some(TerrainGenerationData {
        main_id: main,
        chunk_id: id,
        texture_reads: (read_densities, read_materials),
        atomic_read: read_counters,
    });
    println!("Dispatched voxel generation!");
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
    println!("Finished voxel data generation!");
    // Load the read containers that we passed to the pipeline
    let data = terrain.generating.take().unwrap();

    // Load the actual voxels now
    let voxel_pixels = data.texture_reads.0.fill_vec::<f32>().unwrap();
    let material_pixels = data.texture_reads.1.fill_vec::<veclib::Vector2<u8>>().unwrap();
    
    // Create the voxel data on the heap since it's going to be pretty big
    let voxels = voxel_pixels
        .into_iter()
        .zip(material_pixels.into_iter())
        .map(|(density, material)| Voxel {
            density,
            material_id: material.x,
        }).collect::<Vec<Voxel>>();
    let voxels = voxels.into_boxed_slice();
    let voxel_data = VoxelData(voxels);
    
    let positive = data.atomic_read.get(0).unwrap();
    let negative = data.atomic_read.get(1).unwrap();
    // Check if we have a valid surface that we can create a mesh out of
    let valid_surface = positive > 0 && negative > 0;

    chunk.voxel_data = Some(voxel_data);
    chunk.valid_surface = valid_surface;
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
