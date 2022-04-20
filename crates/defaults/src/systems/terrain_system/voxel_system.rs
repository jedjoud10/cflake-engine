use crate::{
    components::{Chunk, Transform},
    globals::ChunkGenerationState,
};
use world::{
    rendering::{
        advanced::{compute::ComputeShaderExecutionSettings, storages::Buffer},
        basics::uniforms::Uniforms,
        pipeline::Pipeline,
    },
    terrain::{ChunkCoords, CHUNK_SIZE},
    World, ecs::Entity,
};

// Simply run the compute shaders for now
fn generate(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut Chunk, entity: Entity) {
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
        uniforms.set_u32("node_depth", chunk.coords.depth as u32);
        uniforms.set_u32("node_size", chunk.coords.size as u32);
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
        uniforms.set_u32("node_depth", chunk.coords.depth as u32);
        uniforms.set_u32("node_size", chunk.coords.size as u32);
        // Clear the atomics then set them
        generator.atomics.set([0, 0, 0, 0]);
        uniforms.set_atomic_group("_", &mut generator.atomics, 0);
        // And execute the shader
        let settings = ComputeShaderExecutionSettings::new(vek::Vec3::new(AXIS2, AXIS2, AXIS2));
        let compute = pipeline.get(&generator.secondary_compute).unwrap();
        compute.run(pipeline, settings, uniforms, true).unwrap();
    });
    terrain.manager.current_chunk_state = ChunkGenerationState::FetchShaderStorages(entity, chunk.coords);
}

// Then, a frame later, fetch the buffer data
fn fetch_buffers(terrain: &mut crate::globals::Terrain, chunk: &mut Chunk, entity: Entity, coords: ChunkCoords) {
    // Get the valid counters
    let generator = &mut terrain.generator;
    let counters = generator.atomics.get();

    // Check if we have a surface or not
    if counters[0] == 0 || counters[1] == 0 {
        // We must manually remove this chunk since we will never be able to generate it's mesh
        terrain.manager.chunks_generating.remove(&coords);
        // Switch states
        terrain.manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(entity, false, None);
        return;
    }
    // We can read from the SSBO now
    let allocated_packed_voxels = &mut generator.packed.0;
    // READ
    generator.ssbo_final_voxels.storage_mut().read(allocated_packed_voxels.as_mut_slice());
    let (id, persistent) = generator.buffer.store(&generator.packed);

    // Switch states
    terrain.manager.current_chunk_state = ChunkGenerationState::EndVoxelDataGeneration(entity, true, Some(id));

    // Save the persistent voxel data inside the chunk
    chunk.persistent = Some(persistent);
}

// The voxel systems' update loop
fn run(world: &mut World) {
    // Get the pipeline without angering the borrow checker
    let terrain = world.globals.get_mut::<crate::globals::Terrain>();
    if let Some(terrain) = terrain {
        // The edit system didn't pack the edits yet, we must skip
        if terrain.editer.is_pending() {
            return;
        }
        
        // Either generate voxel data or fetch voxel data
        if terrain.manager.current_chunk_state == ChunkGenerationState::RequiresVoxelData {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that has the highest priority
            if let Some((entity, _)) = terrain.manager.priority_list.pop() {
                // Start generating some voxel data on the GPU
                let entry = world.ecs.entry(entity).unwrap();
                let chunk = entry.get_mut::<Chunk>().unwrap();
                generate(terrain, &world.pipeline, chunk, entity);
            }
        } else if let ChunkGenerationState::FetchShaderStorages(entity, coords) = terrain.manager.current_chunk_state {
            // We should fetch the shader storages now
            let entry = world.ecs.entry(entity).unwrap();
                let chunk = entry.get_mut::<Chunk>().unwrap();
            fetch_buffers(terrain, chunk, entity, coords);
        }
    }
}

// Create a voxel system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
