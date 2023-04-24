use std::time::Instant;

use crate::{Chunk, ChunkState, Terrain, TerrainMaterial};
use coords::Position;
use ecs::Scene;
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
    let mut tex_coords = world
        .get_mut::<Storage<AttributeBuffer<attributes::TexCoord>>>()
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

    // Get global indexed indirect draw buffer
    let indirect = indirects.get_mut(&manager.indexed_indirect_buffer);

    // Convert "Dirty" chunks into "Pending"
    let query = scene
        .query_mut::<(&mut Chunk, &mut Surface<TerrainMaterial>)>()
        .into_iter();
    for (chunk, surface) in query.filter(|(c, _)| c.state == ChunkState::Dirty) {
        chunk.state = ChunkState::Pending;
        surface.visible = false;

        // Write to the indices the updated ranges if needed
        if let Some(range) = chunk.ranges {
            if range.y > range.x {
                let indices = &mut memory.sub_allocation_chunk_indices[chunk.allocation];
                indices
                    .splat((range.x as usize)..(range.y as usize), u32::MAX)
                    .unwrap();
            }
        }

        // Update indirect buffer
        indirect
            .write(
                &[DrawIndexedIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: 0,
                }],
                chunk.global_index,
            )
            .unwrap();

        chunk.ranges = None;
    }

    // Find the chunk with the highest priority
    let mut vec = scene
        .query_mut::<(
            &mut Chunk,
            &Position,
            &mut Surface<TerrainMaterial>,
            &mut Renderer,
        )>()
        .into_iter()
        .collect::<Vec<_>>();
    vec.sort_by(|(a, _, _, _), (b, _, _, _)| b.priority.total_cmp(&a.priority));

    // Iterate over the chunks that we need to generate
    for (chunk, position, surface, renderer) in vec {
        // Don't generate the voxels and mesh for chunks that had their mesh already generated
        if chunk.state != ChunkState::Pending {
            continue;
        }

        // The renderer is initialized when the mesh get it's surface
        renderer.instant_initialized = Some(std::time::Instant::now());

        // Reset the current counters
        mesher.counters.write(&[0; 2], 0).unwrap();
        memory.offsets.write(&[u32::MAX, u32::MAX], 0).unwrap();

        // Create a compute pass for both the voxel and mesh compute shaders
        let mut pass = ComputePass::begin(&graphics);

        // Create the voxel data and store it in the image
        let mut active = pass.bind_shader(&voxelizer.compute_voxels);

        // Needed since SN only runs for a volume 2 units smaller than a perfect cube
        let factor = (settings.size as f32 - 3.0) / (settings.size as f32);

        // Set the push constants
        active
            .set_push_constants(|x| {
                // Use offset * factor as position offset
                let offset = position.with_w(0.0f32) * factor;
                let offset = GpuPod::into_bytes(&offset);

                // Combine chunk index and allocation into the same vector
                let packed = vek::Vec2::new(chunk.global_index, chunk.allocation).as_::<u32>();
                let time = GpuPod::into_bytes(&packed);

                // Push the bytes to the GPU
                x.push(offset, 0).unwrap();
                x.push(time, offset.len() as u32).unwrap();

                // Call the set group callback
                if let Some(callback) = voxelizer.set_push_constant_callback.as_ref() {
                    (callback)(x);
                }
            })
            .unwrap();

        // One global bind group for voxel generation
        active
            .set_bind_group(0, |set| {
                set.set_storage_texture("voxels", &mut voxelizer.voxels)
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
                set.set_storage_texture("voxels", &mut voxelizer.voxels)
                    .unwrap();
                set.set_storage_texture("cached_indices", &mut mesher.cached_indices)
                    .unwrap();
                set.set_storage_buffer("counters", &mut mesher.counters, ..)
                    .unwrap();
            })
            .unwrap();
        active
            .set_bind_group(1, |set| {
                set.set_storage_buffer("vertices", &mut mesher.temp_vertices, ..)
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
                set.set_storage_texture("cached_indices", &mut mesher.cached_indices)
                    .unwrap();
                set.set_storage_texture("voxels", &mut voxelizer.voxels)
                    .unwrap();
                set.set_storage_buffer("counters", &mut mesher.counters, ..)
                    .unwrap();
            })
            .unwrap();
        active
            .set_bind_group(1, |set| {
                set.set_storage_buffer("triangles", &mut mesher.temp_triangles, ..)
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
                set.set_storage_buffer(
                    "indices",
                    &mut memory.sub_allocation_chunk_indices[chunk.allocation],
                    ..,
                )
                .unwrap();
                set.set_storage_buffer("offsets", &mut memory.offsets, ..)
                    .unwrap();
                set.set_storage_buffer("counters", &mut mesher.counters, ..)
                    .unwrap();
            })
            .unwrap();

        // Get the local chunk index for the current allocation
        active
            .set_push_constants(|x| {
                let index = chunk.local_index;
                let index = index as u32;
                let bytes = GpuPod::into_bytes(&(index));
                x.push(bytes, 0).unwrap();
            })
            .unwrap();

        //let dispatch = (terrain.sub_allocations as f32 / 32 as f32).ceil() as u32;
        active.dispatch(vek::Vec3::new(1, 1, 1)).unwrap();

        // Get the output packed tex coord from resource storage
        let output_vertices = tex_coords.get_mut(&memory.shared_tex_coord_buffers[chunk.allocation]);

        // Get the output triangles from resrouce storage
        let output_triangles = triangles.get_mut(&memory.shared_triangle_buffers[chunk.allocation]);

        // Copy the generated vertex and tri data to the permanent buffer
        let mut active = pass.bind_shader(&memory.compute_copy);
        active
            .set_bind_group(0, |set| {
                set.set_storage_buffer("temporary_vertices", &mut mesher.temp_vertices, ..)
                    .unwrap();
                set.set_storage_buffer("temporary_triangles", &mut mesher.temp_triangles, ..)
                    .unwrap();
                set.set_storage_buffer("offsets", &mut memory.offsets, ..)
                    .unwrap();
                set.set_storage_buffer("counters", &mut mesher.counters, ..)
                    .unwrap();
            })
            .unwrap();
        active
            .set_bind_group(1, |set| {
                set.set_storage_buffer("output_vertices", output_vertices, ..)
                    .unwrap();
                set.set_storage_buffer("output_triangles", output_triangles, ..)
                    .unwrap();
                set.set_storage_buffer("indirect", indirect, ..).unwrap();
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

        // Submit the work to the GPU, and fetch counters and offsets
        let _counters = mesher.counters.as_view(..).unwrap();
        let counters = _counters.to_vec();
        let _offsets = memory.offsets.as_view(..).unwrap();
        let offsets = _offsets.to_vec();

        // Read as vertex and triangle separately
        let vertex_count = counters[0];
        let triangle_count = counters[1];
        let vertices_offset = offsets[0];
        let triangle_indices_offset = offsets[1];

        // Check if we are OOM lol
        if vertices_offset / settings.tex_coords_per_sub_allocation
            != triangle_indices_offset / settings.triangles_per_sub_allocation
        {
            panic!("Out of memory xD MDR");
        }

        // Calculate sub-allocation index and length
        let count = f32::max(
            vertex_count as f32 / settings.tex_coords_per_sub_allocation as f32,
            triangle_count as f32 / settings.triangles_per_sub_allocation as f32,
        );
        let count = count.ceil() as u32;
        let offset = vertices_offset / settings.tex_coords_per_sub_allocation;

        // Update chunk range (if valid) and set visibility
        if count > 0 {
            chunk.ranges = Some(vek::Vec2::new(offset, count + offset));
            surface.visible = true;
        } else {
            chunk.ranges = None;
            surface.visible = false;
        }

        //surface.visible = true;
        chunk.state = ChunkState::Generated;
        return;
    }
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .after(crate::systems::manager::system)
        .before(rendering::systems::rendering::system);
}
