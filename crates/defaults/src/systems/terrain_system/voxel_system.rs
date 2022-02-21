use main::{
    core::World,
    ecs::{entity::EntityID, event::EventKey},
    rendering::{
        advanced::compute::ComputeShaderExecutionSettings,
        basics::{
            buffer_operation::{BufferOperation, ReadBytes, WriteBytes},
            uniforms::SetUniformsCallback,
        },
        object::TrackedTask,
        pipeline::{pipec, Pipeline},
    },
    terrain::{PackedVoxel, CHUNK_SIZE},
};

use crate::globals::ChunkGenerationState;

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    let generator = &mut terrain.voxel_generator;
    // Create the compute shader execution settings and execute the compute shader
    const AXIS: u16 = ((CHUNK_SIZE + 2) as u16) / 8 + 1;
    const AXIS2: u16 = ((CHUNK_SIZE + 1) as u16) / 8 + 1;
    // Set the uniforms for the first compute shader
    let chunk_coords = chunk.coords;
    let arbitrary_voxels = generator.shader_storage_arbitrary_voxels;
    let output_voxels = generator.shader_storage_final_voxels;
    let shader_storage_edits = generator.shader_storage_edits;
    let num_edits = generator.packed_edits_num;
    let atomics = generator.atomics;
    let uniforms = SetUniformsCallback::new(move |uniforms| {
        uniforms.set_shader_storage("arbitrary_voxels", arbitrary_voxels, 0);
        uniforms.set_shader_storage("terrain_edits", shader_storage_edits, 1);
        uniforms.set_vec3f32("node_pos", chunk_coords.position.into());
        uniforms.set_i32("node_size", chunk_coords.size as i32);
        uniforms.set_u32("num_terrain_edits", num_edits as u32);
    });
    // Now we can execute the compute shader and the read bytes command
    let execution_settings = ComputeShaderExecutionSettings::new(veclib::vec3(AXIS, AXIS, AXIS)).with_callback(uniforms);
    // Additional uniforms
    let execution_settings = if let Some(uniforms) = &generator.uniforms {
        execution_settings.with_callback(uniforms.clone())
    } else {
        execution_settings
    };
    pipec::tracked_task(pipeline, TrackedTask::RunComputeShader(generator.compute_shader, execution_settings), generator.compute_id);
    // After we run the first compute shader, we must run the second compute shader, then read from the final SSBO and counters

    // Set the uniforms for the second compute shader
    let uniforms = SetUniformsCallback::new(move |uniforms| {
        uniforms.set_shader_storage("arbitrary_voxels", arbitrary_voxels, 0);
        uniforms.set_shader_storage("output_voxels", output_voxels, 1);
        uniforms.set_vec3f32("node_pos", chunk_coords.position.into());
        uniforms.set_i32("node_size", chunk_coords.size as i32);
        // Set the atomic counters
        uniforms.set_atomic_group("_", atomics, true, 0);
    });
    // And execute the shader
    let execution_settings2 = ComputeShaderExecutionSettings::new(veclib::vec3(AXIS2, AXIS2, AXIS2)).with_callback(uniforms);
    // Additional uniforms
    let execution_settings2 = if let Some(uniforms) = &generator.uniforms {
        execution_settings2.with_callback(uniforms.clone())
    } else {
        execution_settings2
    };
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::RunComputeShader(generator.second_compute_shader, execution_settings2),
        generator.compute_id2,
        generator.compute_id,
    );
    // And also read from the atomic counters
    let read_counters = ReadBytes::default();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::AtomicGroupOp(generator.atomics, BufferOperation::Read(read_counters.clone())),
        generator.read_counters,
        generator.compute_id2,
    );

    // Send a task to read the final voxel shader values
    let read_bytes = ReadBytes::default();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::ShaderStorageOp(generator.shader_storage_final_voxels, BufferOperation::Read(read_bytes.clone())),
        generator.read_final_voxels,
        generator.compute_id2,
    );

    // Store the CPU side readers
    generator.pending_reads = Some((read_counters, read_bytes));
    terrain.chunks_manager.current_chunk_state = ChunkGenerationState::BeginVoxelDataGeneration(id);
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, _pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
    let (read_atomic_counter_bytes, read_voxel_data_bytes) = terrain.voxel_generator.pending_reads.take().unwrap();
    // Get the valid counters
    let mut read_counters = [0_u32; 2];
    read_atomic_counter_bytes.fill_array(&mut read_counters).unwrap();
    let positive = *read_counters.get(0).unwrap();
    let negative = *read_counters.get(1).unwrap();
    let id = *terrain.chunks_manager.current_chunk_state.as_begin_voxel_data_generation().unwrap();
    if positive == 0 || negative == 0 {
        // We must manually remove this chunk since we will never be able to generate it's mesh
        terrain.chunks_manager.chunks_generating.remove(&chunk.coords);
        // Switch states
        terrain.chunks_manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(id, false);
        return;
    }

    // We can read from the SSBO now
    let allocated_packed_voxels = &mut terrain.voxel_generator.packed_chunk_voxel_data.0;
    let arr = allocated_packed_voxels.as_mut_slice();
    read_voxel_data_bytes.fill_array::<PackedVoxel>(arr).unwrap();
    terrain.voxel_generator.stored_chunk_voxel_data.store(&terrain.voxel_generator.packed_chunk_voxel_data);

    // Switch states
    terrain.chunks_manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(id, true);
}
// The voxel systems' update loop
fn run(world: &mut World, mut data: EventKey) {
    let query = data.as_query_mut().unwrap();
    // Get the pipeline without angering the borrow checker
    let pipeline = world.pipeline.read();

    let terrain = world.globals.get_global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // Update the packed edits on the GPU
        if let Some(edits) = terrain.voxel_generator.packed_edits_update.take() {
            // Send a task to read the final voxel shader values
            let write_bytes = WriteBytes::new(edits);
            pipec::tracked_task(
                &pipeline,
                TrackedTask::ShaderStorageOp(terrain.voxel_generator.shader_storage_edits, BufferOperation::Write(write_bytes)),
                terrain.voxel_generator.write_packed_edits,
            );
        }
        // For each chunk in the ter
        if terrain.chunks_manager.current_chunk_state == ChunkGenerationState::RequiresVoxelData {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that has the highest priority
            if let Some((entity_id, _)) = terrain.chunks_manager.priority_list.pop() {
                let mut lock_ = query.write();
                let components = lock_.get_mut(&entity_id).unwrap();
                // We break out at the first chunk if we start generating it's voxel data
                let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
                // We can set our state as not generating if none of the chunks want to generate voxel data
                // We must start generating the voxel data for this chunk
                start_generation(&mut *terrain, &pipeline, &mut *chunk, entity_id);
            }
        } else {
            // We must check if we have finished generating or not
            let generator = &terrain.voxel_generator;
            if pipec::did_tasks_execute(
                &pipeline,
                &[generator.compute_id, generator.compute_id2, generator.read_counters, generator.read_final_voxels],
            ) {
                // We will now update the chunk data to store our new voxel data
                if let ChunkGenerationState::BeginVoxelDataGeneration(id) = terrain.chunks_manager.current_chunk_state {
                    let mut lock_ = query.write();
                    let components = lock_.get_mut(&id).unwrap();
                    // Get our chunk and set it's new data
                    let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
                    finish_generation(&mut *terrain, &*pipeline, &mut *chunk);
                }
            }
        }
    }
}
// Create a voxel system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build();
}
