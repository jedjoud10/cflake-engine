use crate::{Chunk, ChunkState, Terrain};
use assets::Assets;
use coords::{Position, Scale};
use ecs::{Entity, Scene};
use graphics::{
    ActivePipeline, ComputeModule, ComputePass, ComputeShader, GpuPod, Graphics, Texture,
    TriangleBuffer, Vertex,
};
use rendering::{attributes, AttributeBuffer};
use utils::{Storage, Time};
use world::{System, World};

// Look in the world for any chunks that need their mesh generated and generate it
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let _time = world.get::<Time>().unwrap();
    let _terrain = world.get_mut::<Terrain>();

    // If we don't have terrain, don't do shit
    let Ok(mut _terrain) = _terrain else {
        return;
    };

    // Get the required resources from the world
    let terrain = &mut *_terrain;
    let mut scene = world.get_mut::<Scene>().unwrap();
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
    let query = scene.query_mut::<&mut Chunk>().into_iter();
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
        memory.visibility_bitsets[chunk.allocation].remove(chunk.local_index);
    }

    // Find the chunk with the highest priority
    let mut vec = scene
        .query_mut::<(&mut Chunk, &Position, &Scale, &Entity)>()
        .into_iter()
        .collect::<Vec<_>>();
    vec.sort_by(|(a, _, _, _), (b, _, _, _)| {
        a.generation_priority.total_cmp(&b.generation_priority)
    });
    vec.retain(|(chunk, _, _, _)| chunk.state == ChunkState::Pending);
    let Some((chunk, position, scale, _entity)) = vec.pop() else {
        // We have no chunks to generate, check if we should hot reload now blud
        if let Some((rx, _)) = voxelizer.hot_reload.as_ref() {
            if rx.try_recv().is_ok() {
                rx.try_iter().count();
                let assets = world.get::<Assets>().unwrap();
                let graphics = world.get::<Graphics>().unwrap();
                
                // Uncache the old shader so we can force hot reloading
                assets.uncache("engine/shaders/terrain/voxel.glsl");
                
                // Re-load the compute shader and compile it again
                let module = assets
                    .load::<ComputeModule>("engine/shaders/terrain/voxels.comp")
                    .unwrap();
                let compiler = crate::create_compute_voxels_compiler(&assets, &graphics);
                match ComputeShader::new(module, &compiler) {
                    Ok(shader) => {
                        voxelizer.compute_voxels = shader;

                        // Force the regeneration of all chunks
                        let query = scene
                            .query_mut::<&mut Chunk>()
                            .into_iter()
                            .filter(|x| matches!(x.state, ChunkState::Generated { .. }));
                        for x in query {
                            x.regenerate();
                        }
                    },
                    Err(error) => {
                        log::error!("Voxel Shader Error: {:?}", error);
                    },
                }
                
            }
        }

        return;
    };

    let last_chunk_generated = scene
        .query_mut::<(&Chunk, &Entity)>()
        .into_iter()
        .filter(|(chunk, _)| chunk.state == ChunkState::PendingReadbackStart)
        .map(|(_, entity)| *entity)
        .count();

    // I FUCKING TOLD YOU NOT TO GENERATE MORE THAN ONE YOU FUCKING DUMB FUCK
    if last_chunk_generated > 1 {
        return;
    }

    // Get the resources used for this chunk
    let counters = &mut memory.counters;
    let offsets = &mut memory.offsets;
    let suballocations = &mut memory.sub_allocation_chunk_indices[chunk.allocation];
    let indirect = &mut memory.generated_indexed_indirect_buffers[chunk.allocation];
    let size = settings.mesher.size;
    let max_depth = settings.mesher.max_octree_depth;
    let sub_allocation_count = settings.memory.sub_allocation_count;

    // Update alloc-local indirect draw args
    indirect
        .write(
            &[crate::util::DEFAULT_DRAW_INDEXED_INDIRECT],
            chunk.local_index,
        )
        .unwrap();

    // Reset required values
    counters.write(&[0; 2], 0).unwrap();
    let mut view = mesher.cached_indices.view_mut(0).unwrap();
    view.splat(None, u32::MAX).unwrap();
    offsets.write(&[u32::MAX; 2], 0).unwrap();

    /*
    mesher.temp_vertices.splat(.., vek::Vec4::zero()).unwrap();
    mesher.temp_triangles.splat(.., [0; 3]).unwrap();
    */

    // Update alloc-local position buffer
    let packed = (*position).with_w(**scale);
    let buffer = &mut memory.generated_position_scaling_buffers[chunk.allocation];
    buffer.write(&[packed], chunk.local_index).unwrap();

    // Create a compute pass for ALL compute terrain shaders
    let mut pass = ComputePass::begin(&graphics);
    let mut active = pass.bind_shader(&voxelizer.compute_voxels);

    // Needed since SN only runs for a volume 2 units smaller than a perfect cube
    let node = chunk.node.unwrap();
    let factor = (node.size() as f32) / (size as f32 - 4.0);

    // Check if the node has neighbors with different sizes in each direction (bitfield)
    let skirts_directions = crate::find_skirts_direction(&node, &manager.octree);

    // Set the push constants
    active
        .set_push_constants(|x| {
            // WHY DO WE NEED TO MULTIPLY BY 0.5 WHY WHY WHY WHY WHY (it works tho)
            let offset =
                (node.position().as_::<f32>() - vek::Vec3::broadcast(factor) * 0.5).with_w(0.0f32);
            let offset = GpuPod::into_bytes(&offset);

            // Get the scale of the chunk
            let scale = GpuPod::into_bytes(&factor);

            // Calculate quality floating point value
            let _quality = (node.depth()) as f32 / (max_depth as f32);
            let quality = GpuPod::into_bytes(&_quality);

            // Push the bytes to the GPU
            x.push(offset, 0).unwrap();
            x.push(scale, offset.len() as u32).unwrap();
            x.push(quality, scale.len() as u32 + offset.len() as u32)
                .unwrap();
        })
        .unwrap();

    // One global bind group for voxel generation
    active
        .set_bind_group(0, |set| {
            set.set_storage_texture_mut("voxels", &mut voxelizer.voxel_texture)
                .unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::broadcast(size / 8)).unwrap();

    // Execute the vertex generation shader first
    let mut active = pass.bind_shader(&mesher.compute_vertices);

    active
        .set_bind_group(0, |set| {
            set.set_sampled_texture("voxels", &voxelizer.voxel_texture)
                .unwrap();
            set.set_sampler("voxels_sampler", voxelizer.voxel_texture.sampler().unwrap())
                .unwrap();
            set.set_storage_texture_mut("cached_indices", &mut mesher.cached_indices)
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
        .set_push_constants(|pc| {
            // Use the skirts direction bitfield to limit the number of skirts
            let bytes = GpuPod::into_bytes(&skirts_directions);
            pc.push(bytes, 0).unwrap();

            // Calculate skirts threshold floating point value
            let _skirts_threshold = (max_depth - node.depth()) as f32 / (max_depth as f32);
            let skirts_threshold = GpuPod::into_bytes(&_skirts_threshold);
            pc.push(skirts_threshold, bytes.len() as u32).unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::broadcast(size / 8)).unwrap();

    // Execute the quad generation shader second
    let mut active = pass.bind_shader(&mesher.compute_quads);
    active
        .set_bind_group(0, |set| {
            set.set_storage_texture("cached_indices", &mesher.cached_indices)
                .unwrap();
            set.set_storage_texture("voxels", &voxelizer.voxel_texture)
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
    active.dispatch(vek::Vec3::broadcast(size / 8)).unwrap();

    // Run a compute shader that will iterate over the ranges and find a free one
    let mut active = pass.bind_shader(&memory.compute_find);
    active
        .set_bind_group(0, |set| {
            set.set_storage_buffer_mut("indices", suballocations, ..)
                .unwrap();
            set.set_storage_buffer_mut("offsets", offsets, ..).unwrap();
            set.set_storage_buffer("counters", counters, ..).unwrap();
        })
        .unwrap();

    let dispatch = (sub_allocation_count as f32 / (32.0 * 64.0)).ceil() as u32;
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
            set.set_storage_buffer("offsets", offsets, ..).unwrap();
            set.set_storage_buffer("counters", counters, ..).unwrap();
        })
        .unwrap();
    active
        .set_bind_group(1, |set| {
            set.set_storage_buffer_mut("output_vertices", output_vertices, ..)
                .unwrap();
            set.set_storage_buffer_mut("output_triangles", output_triangles, ..)
                .unwrap();
            set.set_storage_buffer_mut("indirect", indirect, ..)
                .unwrap();
        })
        .unwrap();
    active
        .set_push_constants(|x| {
            let index = chunk.local_index as u32;
            let index = GpuPod::into_bytes(&index);
            x.push(index, 0).unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::new(2048, 1, 1)).unwrap();

    drop(active);
    drop(pass);

    // Start computing this sheit on the GPU
    graphics.submit(false);

    // Only one chunk must have this state enabled
    // The terrain will fucking kill itself if there's more than one chunk with this state
    chunk.state = ChunkState::PendingReadbackStart;
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .after(crate::systems::manager::system)
        .before(rendering::systems::rendering::system);
}
