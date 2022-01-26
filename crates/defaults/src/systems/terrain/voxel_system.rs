use main::{
    core::{Context, WriteContext},
    ecs::{component::{ComponentQuery, ComponentID}}, terrain::{ChunkCoords, MAIN_CHUNK_SIZE}, rendering::{advanced::compute::ComputeShaderExecutionSettings, basics::{uniforms::ShaderUniformsGroup, texture::{TextureAccessType, TextureReadBytes}}, pipeline::{pipec, Pipeline}, object::PipelineTrackedTask},
};

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, component_id: ComponentID) {
    // Create the compute shader execution settings and execute the compute shader
    let compute = terrain.compute_shader;
    const AXIS: u16 = (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1;
    let execution_settings = ComputeShaderExecutionSettings::new(compute, (AXIS, AXIS, AXIS));
    // Set the uniforms for the compute shader as well
    let mut group = ShaderUniformsGroup::new();
    group.set_image("density_image", terrain.density_texture, TextureAccessType::WRITE);
    group.set_image("material_image", terrain.material_texture, TextureAccessType::WRITE);

    // Chunk specific uniforms
    let chunk_coords = chunk.coords;
    group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
    group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
    group.set_i32("node_size", chunk_coords.size as i32);
    group.set_i32("depth", chunk_coords.depth as i32);

    // Now we can execute the compute shader and the read bytes command
    let execution = pipec::tracked_task(PipelineTrackedTask::RunComputeShader(compute, execution_settings), None, pipeline);
    // Create this for the next step
    let read_densities = TextureReadBytes::default();
    let read_materials = TextureReadBytes::default();
    
    let read_densities_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.density_texture, read_densities.clone()), Some(execution), pipeline);
    let read_materials_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.material_texture, read_materials.clone()), Some(execution), pipeline);
    
    // Combine the tasks to make a finalizer one
    let main = pipec::tracked_finalizer(vec![execution, read_densities_tracked_id, read_materials_tracked_id], pipeline).unwrap();
    terrain.generating = Some((main, component_id, (read_densities, read_materials)));
    println!("Dispatched voxel generation!");
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
                    start_generation(&mut *terrain, &*pipeline, &mut *chunk, components.get_id::<crate::components::Chunk>().unwrap());
                }
                None
            })
        } else {
            // We must check if we have finished generating or not
            if pipec::has_task_executed(terrain.generating.as_ref().unwrap().0, &*pipeline).unwrap() {
                // We finished generating the voxel data for a specific chunk, and we must store it now.
                // To find the specified chunk, we will loop through every chunk and get the one with the specified 
                println!("Finished voxel data generation!");
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
