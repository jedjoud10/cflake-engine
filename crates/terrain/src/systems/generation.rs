use std::time::Instant;

use crate::{Chunk, ChunkState, Terrain, TerrainMaterial};
use coords::{Position, Scale};
use ecs::{Scene, Entity};
use graphics::{
    ActivePipeline, ComputePass, DrawIndexedIndirect, DrawIndexedIndirectBuffer, GpuPod, Graphics,
    TriangleBuffer, Vertex,
};
use rendering::{attributes, AttributeBuffer, IndirectMesh, Renderer, Surface};
use utils::{Storage, Time};
use world::{System, World};

// Look in the world for any chunks that need their mesh generated and generate it
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let time = world.get::<Time>().unwrap();
    let _terrain = world.get_mut::<Terrain>();

    // If we don't have terrain, don't do shit
    let Ok(mut _terrain) = _terrain else {
        return;
    };

    // Get the required resources from the world
    let terrain = &mut *_terrain;
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut indirects = world
        .get_mut::<Storage<DrawIndexedIndirectBuffer>>()
        .unwrap();
    let mut positions = world
        .get_mut::<Storage<AttributeBuffer<attributes::Position>>>()
        .unwrap();
    let mut triangles = world.get_mut::<Storage<TriangleBuffer<u32>>>().unwrap();

    // Get the required sub-resources from the terrain resource
    let (manager, voxelizer, mesher, memory, settings) = (
        &mut terrain.manager,
        &mut terrain.voxelizer,
        &mut terrain.mesher,
        &mut terrain.memory,
        &mut terrain.settings,
    );

    // Convert "Dirty" chunks into "Pending", and clears the old memory used by those chunks
    let query = scene
        .query_mut::<&mut Chunk>()
        .into_iter();
    for chunk in query.filter(|c| c.state == ChunkState::Dirty) {
        chunk.state = ChunkState::Pending;
        
        // Write to the indices the updated ranges if needed
        if let Some(range) = chunk.ranges {
            if range.y > range.x {
                let indices = &mut memory.sub_allocation_chunk_indices[chunk.allocation];
                indices
                    .splat((range.x as usize)..(range.y as usize), u32::MAX)
                    .unwrap();
            }
        }

        // Remove the chunk CPU range
        chunk.ranges = None;
    }

    // Find the chunk with the highest priority
    let mut vec = scene
        .query_mut::<(
            &mut Chunk,
            &Position,
            &Scale,
            &Entity,
        )>()
        .into_iter()
        .collect::<Vec<_>>();
    vec.sort_by(|(a, _, _, _), (b, _, _, _)| a.generation_priority.total_cmp(&b.generation_priority));
    vec.retain(|(chunk, _, _, _)| chunk.state == ChunkState::Pending);
    let Some((chunk, position, scale, entity)) = vec.pop() else {
        manager.last_chunk_generated = None;
        return;
    };

    // NEEDED FOR ASYNC READBACK
    let index = time.frame_count() as usize % 2;

    // Get the resources used for this chunk
    let voxels = &mut voxelizer.voxel_textures[index];
    let counters = &mut memory.counters[index];
    let offsets = &mut memory.offsets[index];
    let indices = &mut mesher.cached_indices;
    let suballocations = &mut memory.sub_allocation_chunk_indices[chunk.allocation];
    let indirect = &mut memory.generated_indexed_indirect_buffer;

    // Reset required values
    counters.write(&[0; 2], 0).unwrap();
    offsets.write(&[u32::MAX; 2], 0).unwrap();
    indirect.write(&[crate::util::DEFAULT_DRAW_INDEXED_INDIRECT], chunk.global_index).unwrap();

    // Update position buffer
    let packed = (*position).with_w(**scale);
    let buffer = &mut manager.position_scaling_buffer;
    buffer.write(&[packed], chunk.global_index).unwrap();

    // Create a compute pass for ALL compute terrain shaders
    let mut pass = ComputePass::begin(&graphics);
    let mut active = pass.bind_shader(&voxelizer.compute_voxels);

    // Needed since SN only runs for a volume 2 units smaller than a perfect cube
    let node = chunk.node.unwrap();
    let factor = (node.size() as f32) / (settings.size as f32 - 3.0);

    // Set the push constants
    active
        .set_push_constants(|x| {
            // WHY DO WE NEED TO MULTIPLY BY 0.5 WHY WHY WHY WHY WHY (it works tho)
            let offset = (node.position().as_::<f32>() - vek::Vec3::broadcast(factor) * 0.5).with_w(0.0f32);
            let offset = GpuPod::into_bytes(&offset);

            // Get the scale of the chunk
            let scale = GpuPod::into_bytes(&factor);

            // Calculate quality index
            let _quality = 4 - ((settings.max_depth - node.depth()).min(4));
            let quality = GpuPod::into_bytes(&_quality);

            // Push the bytes to the GPU
            x.push(offset, 0).unwrap();
            x.push(scale, offset.len() as u32).unwrap();
            x.push(quality, scale.len() as u32 + offset.len() as u32).unwrap();

            // Call the set group callback
            if let Some(callback) = voxelizer.set_push_constant_callback.as_ref() {
                (callback)(x);
            }
        })
        .unwrap();

    // One global bind group for voxel generation
    active
        .set_bind_group(0, |set| {
            set.set_storage_texture_mut("voxels", voxels)
                .unwrap();

            // Call the set group callback
            if let Some(callback) = voxelizer.set_bind_group_callback.as_ref() {
                (callback)(set);
            }
        })
        .unwrap();
    active
        .dispatch(vek::Vec3::broadcast(settings.size / 4))
        .unwrap();

    // Execute the vertex generation shader first
    let mut active = pass.bind_shader(&mesher.compute_vertices);

    active
        .set_bind_group(0, |set| {
            set.set_storage_texture("voxels", voxels)
                .unwrap();
            set.set_storage_texture_mut("cached_indices", indices)
                .unwrap();
            set.set_storage_buffer_mut("counters", counters, ..)
                .unwrap();
        })
        .unwrap();
    active
        .set_bind_group(1, |set| {
            set.set_storage_buffer_mut("vertices", &mut mesher.temp_vertices, ..)
                .unwrap();
        })
        .unwrap();
    active
        .dispatch(vek::Vec3::broadcast(settings.size / 4))
        .unwrap();

    // Execute the quad generation shader second
    let mut active = pass.bind_shader(&mesher.compute_quads);
    active
        .set_bind_group(0, |set| {
            set.set_storage_texture("cached_indices", indices)
                .unwrap();
            set.set_storage_texture("voxels", voxels)
                .unwrap();
            set.set_storage_buffer_mut("counters", counters, ..)
                .unwrap();
        })
        .unwrap();
    active
        .set_bind_group(1, |set| {
            set.set_storage_buffer_mut("triangles", &mut mesher.temp_triangles, ..)
                .unwrap();
        })
        .unwrap();
    active
        .dispatch(vek::Vec3::broadcast(settings.size / 4))
        .unwrap();

    // Run a compute shader that will iterate over the ranges and find a free one
    let mut active = pass.bind_shader(&memory.compute_find);
    active
        .set_bind_group(0, |set| {
            set.set_storage_buffer_mut(
                "indices",
                suballocations,
                ..,
            )
            .unwrap();
            set.set_storage_buffer_mut("offsets", offsets, ..)
                .unwrap();
            set.set_storage_buffer("counters", counters, ..)
                .unwrap();
        })
        .unwrap();

    let dispatch = (settings.sub_allocation_count as f32 / (32.0 * 32.0)).ceil() as u32;
    active.dispatch(vek::Vec3::new(dispatch, 1, 1)).unwrap();

    // Get the output packed tex coord from resource storage
    let output_vertices = positions.get_mut(&memory.shared_positions_buffers[chunk.allocation]);

    // Get the output triangles from resrouce storage
    let output_triangles = triangles.get_mut(&memory.shared_triangle_buffers[chunk.allocation]);

    // Copy the generated vertex and tri data to the permanent buffer
    let mut active = pass.bind_shader(&memory.compute_copy);
    active 
        .set_bind_group(0, |set| {
            set.set_storage_buffer("temporary_vertices", &mesher.temp_vertices, ..)
                .unwrap();
            set.set_storage_buffer("temporary_triangles", &mesher.temp_triangles, ..)
                .unwrap();
            set.set_storage_buffer("offsets", offsets, ..)
                .unwrap();
            set.set_storage_buffer("counters", counters, ..)
                .unwrap();
        })
        .unwrap();
    active
        .set_bind_group(1, |set| {
            set.set_storage_buffer_mut("output_vertices", output_vertices, ..)
                .unwrap();
            set.set_storage_buffer_mut("output_triangles", output_triangles, ..)
                .unwrap();
            set.set_storage_buffer_mut("indirect", indirect, ..).unwrap();
        })
        .unwrap();
    active
        .set_push_constants(|x| {
            let index = chunk.global_index as u32;
            let index = GpuPod::into_bytes(&index);
            x.push(index, 0).unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::new(2048, 1, 1)).unwrap();

    drop(active);
    drop(pass);
    
    // Show the chunk using the temporary visibility vector
    manager.visibility_bitset.set(chunk.global_index);
    
    // Start computing this sheit on the GPU
    graphics.submit(false);
    chunk.state = ChunkState::Generated;
    manager.last_chunk_generated = Some(*entity);
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .after(crate::systems::manager::system)
        .before(rendering::systems::rendering::system);
}
