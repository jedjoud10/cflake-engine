

use std::time::Instant;

use coords::Position;
use ecs::{Scene};
use graphics::{
    ComputePass, DrawIndexedIndirectBuffer,
    GpuPod, Graphics, TriangleBuffer, Vertex, ActivePipeline,
};
use rendering::{
    attributes, AttributeBuffer, IndirectMesh, Surface, Renderer,
};
use utils::{Storage, Time};
use world::{System, World};
use crate::{Chunk, ChunkState, Terrain, TerrainMaterial};

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
    let mut indirects = world
        .get_mut::<Storage<DrawIndexedIndirectBuffer>>()
        .unwrap();
    let mut vertices = world
        .get_mut::<Storage<AttributeBuffer<attributes::Position>>>()
        .unwrap();
    let mut triangles =
        world.get_mut::<Storage<TriangleBuffer<u32>>>().unwrap();
    let meshes =
        world.get_mut::<Storage<IndirectMesh>>().unwrap();

    // Get the required sub-resources from the terrain resource
    let (voxelizer, mesher, memory, settings) = (
        &mut terrain.voxelizer,
        &mut terrain.mesher,
        &mut terrain.memory,
        &mut terrain.settings,
    );

    // Find the closest chunk to the camera 
    let mut vec = scene.query_mut::<(
        &mut Chunk,
        &Position,
        &mut Surface<TerrainMaterial>,
        &mut Renderer,
    )>().into_iter().collect::<Vec<_>>();
    vec.sort_by(|(a, _, _, _), (b, _, _, _)| a.priority.total_cmp(&b.priority));

    // Iterate over the chunks that we need to generate
    for (chunk, position, surface, renderer) in vec {
        // Don't generate the voxels and mesh for culled chunks or chunks that had
        // their mesh already generated
        if surface.culled || chunk.state != ChunkState::Pending {
            continue;
        }

        renderer.instant_initialized = Some(std::time::Instant::now());

        // Get the mesh that is used by this chunk
        let mesh = meshes.get(&surface.mesh);
        let indirect = mesh.indirect();
        let indirect = indirects.get_mut(indirect);

        // Reset the current counters
        mesher.counters.write(&[0; 2], 0).unwrap();
        memory.offsets.write(&[u32::MAX, u32::MAX], 0).unwrap();

        // Create a compute pass for both the voxel and mesh compute shaders
        let mut pass = ComputePass::begin(&graphics);

        // Create the voxel data and store it in the image
        let mut active = pass.bind_shader(&voxelizer.compute_voxels);

        // Needed since SN only runs for a volume 2 units smaller than a perfect cube
        let factor = (settings.size as f32 - 2.0) / (settings.size as f32);

        // Set the push constants
        active
            .set_push_constants(|x| {
                // Use offset * factor as position offset
                let offset = position.with_w(0.0f32) * factor;
                let offset = GpuPod::into_bytes(&offset);

                // Combine chunk index and allocation into the same vector 
                let packed =
                    vek::Vec2::new(chunk.local_index, chunk.allocation)
                        .as_::<u32>();
                let time = GpuPod::into_bytes(&packed);

                // Push the bytes to the GPU
                x.push(offset, 0).unwrap();
                x.push(time,offset.len() as u32).unwrap();
            })
            .unwrap();

        // One global bind group for voxel generation
        active.set_bind_group(0, |set| {
            set.set_storage_texture("voxels", &mut voxelizer.voxels)
                .unwrap();

            // Call the set group callback
            (voxelizer.set_group_callback)(set);
        });
        active.dispatch(vek::Vec3::broadcast(settings.size / 4));

        // Execute the vertex generation shader first
        let mut active = pass.bind_shader(&mesher.compute_vertices);

        active.set_bind_group(0, |set| {
            set.set_storage_texture("voxels", &mut voxelizer.voxels)
                .unwrap();
            set.set_storage_texture(
                "cached_indices",
                &mut mesher.cached_indices,
            )
            .unwrap();
            set.set_storage_buffer("counters", &mut mesher.counters, ..)
                .unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("vertices", &mut mesher.temp_vertices, ..)
                .unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(settings.size / 4));

        // Execute the quad generation shader second
        let mut active = pass.bind_shader(&mesher.compute_quads);
        active.set_bind_group(0, |set| {
            set.set_storage_texture(
                "cached_indices",
                &mut mesher.cached_indices,
            )
            .unwrap();
            set.set_storage_texture("voxels", &mut voxelizer.voxels)
                .unwrap();
            set.set_storage_buffer("counters", &mut mesher.counters, ..)
                .unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("triangles", &mut mesher.temp_triangles, ..)
                .unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(settings.size / 4));
        
        // Run a compute shader that will iterate over the ranges and find a free one
        let mut active = pass.bind_shader(&memory.compute_find);
        active.set_bind_group(0, |set| {
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
        });

        // Get the local chunk index for the current allocation 
        active
            .set_push_constants(|x| {
                let index = chunk.local_index;
                let index = index as u32;
                let bytes= GpuPod::into_bytes(&(index));
                x.push(bytes, 0).unwrap();
            })
            .unwrap();

        //let dispatch = (terrain.sub_allocations as f32 / 32 as f32).ceil() as u32;
        active.dispatch(vek::Vec3::new(1, 1, 1));

        // Get the output vertices from resource storage
        let output_vertices = vertices.get_mut(
            &memory.shared_vertex_buffers[chunk.allocation]
        );

        // Get the output triangles from resrouce storage
        let output_triangles = triangles.get_mut(
            &memory.shared_triangle_buffers[chunk.allocation]
        );

        // Copy the generated vertex and tri data to the permanent buffer
        let mut active = pass.bind_shader(&memory.compute_copy);
        active.set_bind_group(0, |set| {
            set.set_storage_buffer(
                "temporary_vertices",
                &mut mesher.temp_vertices,
                ..,
            )
            .unwrap();
            set.set_storage_buffer(
                "temporary_triangles",
                &mut mesher.temp_triangles,
                ..,
            )
            .unwrap();
            set.set_storage_buffer("offsets", &mut memory.offsets, ..)
                .unwrap();
            set.set_storage_buffer("counters", &mut mesher.counters, ..)
                .unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer(
                "output_vertices",
                output_vertices,
                ..,
            )
            .unwrap();
            set.set_storage_buffer(
                "output_triangles",
                output_triangles,
                ..,
            )
            .unwrap();
            set.set_storage_buffer("indirect", indirect, ..).unwrap();
        });
        active
            .set_push_constants(|x| {
                let index = mesh.offset() as u32;
                let index = GpuPod::into_bytes(&index);
                x.push(index, 0).unwrap();
            })
            .unwrap();
        active.dispatch(vek::Vec3::new(2048, 1, 1));
        
        drop(active);
        drop(pass);

        // Submit the work to the GPU, and fetch counters and offsets
        graphics.submit(true);
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
        if vertices_offset / settings.vertices_per_sub_allocation != triangle_indices_offset / settings.triangles_per_sub_allocation {
            panic!("Out of memory xD MDR");
        }
        
        // Calculate sub-allocation index and length
        let count = f32::max(vertex_count as f32 / settings.vertices_per_sub_allocation as f32, triangle_count as f32 / settings.triangles_per_sub_allocation as f32); 
        let count = count.ceil() as u32;
        let offset = vertices_offset / settings.vertices_per_sub_allocation;
        
        // Update chunk range
        chunk.ranges = Some(vek::Vec2::new(offset, count + offset));

        // Make the surface visible and set it's state
        surface.visible = true;
        chunk.state = ChunkState::Generated;
        return;
    }
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}