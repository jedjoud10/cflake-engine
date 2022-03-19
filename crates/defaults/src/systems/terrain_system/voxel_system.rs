use crate::{
    components::{Chunk, Transform},
    globals::ChunkGenerationState,
};
use world::{
    ecs::{
        component::{ComponentQueryParams, ComponentQuerySet},
        entity::EntityKey,
    },
    rendering::{
        advanced::{compute::ComputeShaderExecutionSettings, storages::Buffer},
        basics::uniforms::Uniforms,
        pipeline::Pipeline,
    },
    terrain::{ChunkCoords, CHUNK_SIZE},
    World,
};

// Simply run the compute shaders for now
fn generate(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut Chunk, key: EntityKey) {
    let generator = &mut terrain.generator;
    // Create the compute shader execution settings and execute the compute shader
    const AXIS: u16 = ((CHUNK_SIZE + 2) as u16) / 8 + 1;
    const AXIS2: u16 = ((CHUNK_SIZE + 1) as u16) / 8 + 1;
    // Set the uniforms for the first compute shader
    let program = pipeline.get(&generator.primary_compute).unwrap().program();
    Uniforms::new(program, pipeline, |mut uniforms| {
        uniforms.set_shader_storage("arbitrary_voxels", &mut generator.ssbo_voxels, 0);
        uniforms.set_shader_storage("terrain_edits", &mut generator.ssbo_edits, 1);
        uniforms.set_vec3f32("node_pos", chunk.coords.position.as_());
        uniforms.set_i32("node_size", chunk.coords.size as i32);
        uniforms.set_u32("num_terrain_edits", generator.ssbo_edits.storage().len() as u32);
        // Now we can execute the compute shader and the read bytes command
        let settings = ComputeShaderExecutionSettings::new(vek::Vec3::new(AXIS, AXIS, AXIS));
        let compute = pipeline.get(&generator.primary_compute).unwrap();
        compute.run(pipeline, settings, uniforms, true).unwrap();
    });    

    // Set the uniforms for the second compute shader
    let program = pipeline.get(&generator.secondary_compute).unwrap().program();
    Uniforms::new(program, pipeline, |mut uniforms| {
        uniforms.set_shader_storage("arbitrary_voxels", &mut generator.ssbo_voxels, 0);
        uniforms.set_shader_storage("output_voxels", &mut generator.ssbo_final_voxels, 1);
        uniforms.set_vec3f32("node_pos", chunk.coords.position.as_());
        uniforms.set_i32("node_size", chunk.coords.size as i32);
        // Clear the atomics then set them
        generator.atomics.set([0, 0, 0, 0]);
        uniforms.set_atomic_group("_", &mut generator.atomics, 0);
        // And execute the shader
        let settings = ComputeShaderExecutionSettings::new(vek::Vec3::new(AXIS2, AXIS2, AXIS2));
        let compute = pipeline.get(&generator.secondary_compute).unwrap();
        compute.run(pipeline, settings, uniforms, true).unwrap();
    });
    terrain.manager.current_chunk_state = ChunkGenerationState::FetchShaderStorages(key, chunk.coords);
}

// Then, a frame later, fetch the buffer data
fn fetch_buffers(terrain: &mut crate::globals::Terrain, key: EntityKey, coords: ChunkCoords) {
    // READ
    // Get the valid counters
    let generator = &mut terrain.generator;
    let read_counters = generator.atomics.get();
    let positive = *read_counters.get(0).unwrap();
    let negative = *read_counters.get(1).unwrap();
    if positive == 0 || negative == 0 {
        // We must manually remove this chunk since we will never be able to generate it's mesh
        terrain.manager.chunks_generating.remove(&coords);
        // Switch states
        terrain.manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(key, false, None);
        return;
    }
    // We can read from the SSBO now
    let allocated_packed_voxels = &mut generator.packed.0;
    // READ
    generator.ssbo_final_voxels.storage_mut().read(allocated_packed_voxels.as_mut_slice());
    let id = generator.buffer.store(&generator.packed);

    // Switch states
    terrain.manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(key, true, Some(id));
}

// The voxel systems' update loop
fn run(world: &mut World, mut data: ComponentQuerySet) {
    let query = &mut data.get_mut(0).unwrap().all;
    // Get the pipeline without angering the borrow checker
    let terrain = world.globals.get_mut::<crate::globals::Terrain>();
    if let Ok(terrain) = terrain {
        // The edit system didn't pack the edits yet, we must skip
        if terrain.editer.is_pending() {
            return;
        }
        // For each chunk in the terrain
        if terrain.manager.current_chunk_state == ChunkGenerationState::RequiresVoxelData {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that has the highest priority
            if let Some((key, _)) = terrain.manager.priority_list.pop() {
                let lock_ = query;
                let components = lock_.get_mut(&key).unwrap();
                // We break out at the first chunk if we start generating it's voxel data
                let chunk = components.get_mut::<Chunk>().unwrap();
                // We can set our state as not generating if none of the chunks want to generate voxel data
                // We must start generating the voxel data for this chunk
                generate(terrain, &world.pipeline, chunk, key);
            }
        } else if let ChunkGenerationState::FetchShaderStorages(key, coords) = terrain.manager.current_chunk_state {
            // We should fetch the shader storages now
            fetch_buffers(terrain, key, coords);
        }
    }
}

// Create a voxel system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder(&mut world.events.ecs)
        .event(run)
        .query(ComponentQueryParams::default().link::<Transform>().link::<Chunk>())
        .build()
        .unwrap();
}
