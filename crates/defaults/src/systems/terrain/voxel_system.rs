use main::{
    core::{Context, WriteContext},
    ecs::{entity::EntityID, event::EventKey},
    rendering::{
        advanced::{atomic::AtomicGroupRead, compute::ComputeShaderExecutionSettings},
        basics::{readwrite::ReadBytes, transfer::Transferable, uniforms::ShaderUniformsGroup},
        pipeline::{pipec, Pipeline}, object::TrackedTask,
    },
    terrain::{PackedVoxel, CHUNK_SIZE},
};

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    terrain.chunk_id = Some(id);
    // Create the compute shader execution settings and execute the compute shader
    const AXIS: u16 = ((CHUNK_SIZE + 1) as u16).div_ceil(8);
    // Set the uniforms for the first compute shader
    let mut group = ShaderUniformsGroup::new();
    // Chunk specific uniforms
    group.set_shader_storage("arbitrary_voxels", terrain.shader_storage_arbitrary_voxels, 1);
    let chunk_coords = chunk.coords;
    group.set_vec3f32("node_pos", chunk_coords.position.into());
    group.set_i32("node_size", chunk_coords.size as i32);
    let group = ShaderUniformsGroup::combine(group, terrain.custom_uniforms.clone());

    // Now we can execute the compute shader and the read bytes command
    let execution_settings = ComputeShaderExecutionSettings::new((AXIS + 1, AXIS + 1, AXIS + 1)).set_uniforms(group);
    pipec::tracked_task(
        pipeline,
        TrackedTask::RunComputeShader(terrain.compute_shader, execution_settings),
        terrain.compute_id,
    );
    // After we run the first compute shader, we must run the second compute shader, then read from the final SSBO and counters

    // Set the uniforms for the second compute shader
    let mut group = ShaderUniformsGroup::new();
    // Set the atomic counters
    group.set_atomic_group("_", terrain.atomics, 0);
    // Chunk specific uniforms
    group.set_shader_storage("arbitrary_voxels", terrain.shader_storage_arbitrary_voxels, 0);
    group.set_shader_storage("output_voxels", terrain.shader_storage_final_voxels, 1);
    group.set_vec3f32("node_pos", chunk_coords.position.into());
    group.set_i32("node_size", chunk_coords.size as i32);
    let group = ShaderUniformsGroup::combine(group, terrain.custom_uniforms.clone());

    // And execute the shader
    let execution_settings2 = ComputeShaderExecutionSettings::new((AXIS, AXIS, AXIS)).set_uniforms(group);
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::RunComputeShader(terrain.second_compute_shader, execution_settings2),
        terrain.compute_id2,
        terrain.compute_id,
    );
    // And also read from the atomic counters
    let read_counters = AtomicGroupRead::default();
    let read_counters_transfer = read_counters.transfer();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::AtomicGroupRead(terrain.atomics, read_counters_transfer),
        terrain.read_counters,
        terrain.compute_id2,
    );

    // Send a task to read the final voxel shader values
    let read_bytes = ReadBytes::default();
    let read_bytes_transfer = read_bytes.transfer();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::ShaderStorageReadBytes(terrain.shader_storage_final_voxels, read_bytes_transfer),
        terrain.read_final_voxels,
        terrain.compute_id2,
    );

    // Store the CPU side readers
    terrain.cpu_data = Some((read_counters, read_bytes));
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, _pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
    let id = terrain.chunk_id.take().unwrap();
    chunk.pending_voxels = false;
    let (read_counters, read_bytes) = terrain.cpu_data.take().unwrap();
    // Get the valid counters
    let positive = read_counters.get(0).unwrap();
    let negative = read_counters.get(1).unwrap();
    if positive == 0 || negative == 0 {
        // We must manually remove this chunk since we will never be able to generate it's mesh
        terrain.chunks_generating.remove(&chunk.coords);
        return;
    }

    // We can read from the SSBO now
    let allocated_packed_voxels = &mut terrain.packed_chunk_voxel_data.0;
    let arr = allocated_packed_voxels.as_mut_slice();
    read_bytes.fill_array::<PackedVoxel>(arr).unwrap();
    terrain.stored_chunk_voxel_data.store(&terrain.packed_chunk_voxel_data);
    terrain.mesh_gen_chunk_id = Some(id);
    chunk.pending_model = true;
}

// The voxel systems' update loop
fn run(context: &mut Context, data: EventKey) {
    let (query, mut global_fetcher) = data.decompose().unwrap();
    let mut write = context.write().unwrap();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read();

    let terrain = write.ecs.get_global_mut::<crate::globals::Terrain>(&mut global_fetcher);
    if let Ok(mut terrain) = terrain {
        // For each chunk in the terrain, we must create it's respective voxel data, if possible
        if terrain.cpu_data.is_none() {
            if terrain.mesh_gen_chunk_id.is_some() {
                return;
            }
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that has the highest priority
            if let Some((chunk, _)) = terrain.sorted_chunks_generating.pop() {
                query.update(chunk, |components| {
                    // We break out at the first chunk if we start generating it's voxel data
                    let entity_id = components.get_entity_id().unwrap();
                    let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
                    // We can set our state as not generating if none of the chunks want to generate voxel data
                    if !chunk.pending_model && chunk.pending_voxels {
                        // We must start generating the voxel data for this chunk
                        start_generation(&mut *terrain, &pipeline, &mut *chunk, entity_id);
                    }
                });
            }
        } else {
            // We must check if we have finished generating or not
            if pipec::did_tasks_execute(&pipeline, &[terrain.compute_id, terrain.compute_id2, terrain.read_counters, terrain.read_final_voxels]) {
                // We will now update the chunk data to store our new voxel data
                let id = terrain.chunk_id.unwrap();
                query.update(id, |components| {
                    // Get our chunk and set it's new data
                    let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
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
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build();
}
