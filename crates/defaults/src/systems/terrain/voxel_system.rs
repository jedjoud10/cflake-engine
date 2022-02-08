use main::{
    core::World,
    ecs::{entity::EntityID, event::EventKey},
    rendering::{
        advanced::{atomic::AtomicGroupRead, compute::ComputeShaderExecutionSettings},
        basics::{readwrite::ReadBytes, transfer::Transferable, uniforms::SetUniformsCallback},
        object::TrackedTask,
        pipeline::{pipec, Pipeline},
    },
    terrain::{PackedVoxel, CHUNK_SIZE},
};

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    let generator = &mut terrain.generator;
    terrain.chunk_handler.chunk_id = Some(id);
    // Create the compute shader execution settings and execute the compute shader
    const AXIS: u16 = ((CHUNK_SIZE + 1) as u16).div_ceil(8);
    // Set the uniforms for the first compute shader
    let chunk_coords = chunk.coords;
    let arbitrary_voxels = generator.shader_storage_arbitrary_voxels;
    let output_voxels = generator.shader_storage_final_voxels;
    let atomics = generator.atomics;
    let uniforms = SetUniformsCallback::new(move |uniforms| {
        uniforms.set_shader_storage("arbitrary_voxels", arbitrary_voxels, 1);
        uniforms.set_vec3f32("node_pos", chunk_coords.position.into());
        uniforms.set_i32("node_size", chunk_coords.size as i32);
    });
    // Now we can execute the compute shader and the read bytes command
    let execution_settings = ComputeShaderExecutionSettings {
        axii: (AXIS + 1, AXIS + 1, AXIS + 1),
        callback: uniforms,
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
    let execution_settings2 = ComputeShaderExecutionSettings {
        axii: (AXIS, AXIS, AXIS),
        callback: uniforms,
    };
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::RunComputeShader(generator.second_compute_shader, execution_settings2),
        generator.compute_id2,
        generator.compute_id,
    );
    // And also read from the atomic counters
    let read_counters = AtomicGroupRead::default();
    let read_counters_transfer = read_counters.transfer();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::AtomicGroupRead(generator.atomics, read_counters_transfer),
        generator.read_counters,
        generator.compute_id2,
    );

    // Send a task to read the final voxel shader values
    let read_bytes = ReadBytes::default();
    let read_bytes_transfer = read_bytes.transfer();
    pipec::tracked_task_requirement(
        pipeline,
        TrackedTask::ShaderStorageReadBytes(generator.shader_storage_final_voxels, read_bytes_transfer),
        generator.read_final_voxels,
        generator.compute_id2,
    );

    // Store the CPU side readers
    generator.cpu_data = Some((read_counters, read_bytes));
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, _pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
    let id = terrain.chunk_handler.chunk_id.take().unwrap();
    chunk.pending_voxels = false;
    let (read_counters, read_bytes) = terrain.generator.cpu_data.take().unwrap();
    // Get the valid counters
    let positive = read_counters.get(0).unwrap();
    let negative = read_counters.get(1).unwrap();
    if positive == 0 || negative == 0 {
        // We must manually remove this chunk since we will never be able to generate it's mesh
        terrain.chunk_handler.chunks_generating.remove(&chunk.coords);
        return;
    }

    // We can read from the SSBO now
    let allocated_packed_voxels = &mut terrain.generator.packed_chunk_voxel_data.0;
    let arr = allocated_packed_voxels.as_mut_slice();
    read_bytes.fill_array::<PackedVoxel>(arr).unwrap();
    terrain.generator.stored_chunk_voxel_data.store(&terrain.generator.packed_chunk_voxel_data);
    terrain.chunk_handler.mesh_gen_chunk_id = Some(id);
    chunk.pending_model = true;
}

// The voxel systems' update loop
fn run(world: &mut World, data: EventKey) {
    let mut query = data.get_query().unwrap();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = world.pipeline.clone();
    let pipeline = pipeline_.read();

    let terrain = world.globals.get_global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk in the terrain, we must create it's respective voxel data, if possible
        if terrain.generator.cpu_data.is_none() {
            if terrain.chunk_handler.mesh_gen_chunk_id.is_some() {
                return;
            }
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that has the highest priority
            if let Some((entity_id, _)) = terrain.chunk_handler.sorted_chunks_generating.pop() {
                let mut lock_ = query.lock();
                let components = lock_.get_mut(&entity_id).unwrap();
                // We break out at the first chunk if we start generating it's voxel data
                let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
                // We can set our state as not generating if none of the chunks want to generate voxel data
                if !chunk.pending_model && chunk.pending_voxels {
                    // We must start generating the voxel data for this chunk
                    start_generation(&mut *terrain, &pipeline, &mut *chunk, entity_id);
                }
            }
        } else {
            // We must check if we have finished generating or not
            let generator = &terrain.generator;
            if pipec::did_tasks_execute(
                &pipeline,
                &[generator.compute_id, generator.compute_id2, generator.read_counters, generator.read_final_voxels],
            ) {
                // We will now update the chunk data to store our new voxel data
                let id = terrain.chunk_handler.chunk_id.unwrap();
                let mut lock_ = query.lock();
                let components = lock_.get_mut(&id).unwrap();
                // Get our chunk and set it's new data
                let mut chunk = components.get_component_mut::<crate::components::Chunk>().unwrap();
                finish_generation(&mut *terrain, &*pipeline, &mut *chunk);
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
