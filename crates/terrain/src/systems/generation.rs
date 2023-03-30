use ecs::{Position, Scene};
use graphics::{
    ComputePass, DrawIndexedIndirect, DrawIndexedIndirectBuffer,
    GpuPod, Graphics, TriangleBuffer,
};
use rendering::{attributes, Mesh, Surface, IndirectMesh, AttributeBuffer};
use utils::{Storage, Time};
use world::{System, World};

use crate::{Chunk, ChunkState, Terrain, TerrainMaterial};

// Look in the world for any chunks that need their mesh generated and generate it
fn update(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let time = world.get::<Time>().unwrap();
    let _terrain = world.get_mut::<Terrain>();

    // If we don't have terrain, don't do shit
    let Ok(mut _terrain) = _terrain else {
        return;
    };

    let terrain = &mut *_terrain;
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut indirects = world
        .get_mut::<Storage<DrawIndexedIndirectBuffer>>()
        .unwrap();
    let mut vertices = world.get_mut::<Storage<AttributeBuffer<attributes::Position>>>().unwrap();
    let mut triangles = world.get_mut::<Storage<TriangleBuffer<u32>>>().unwrap();
    let mut meshes = world.get_mut::<Storage<IndirectMesh>>().unwrap();

    // Iterate over the chunks that we need to generate
    for (chunk, position, surface) in scene.query_mut::<(
        &mut Chunk,
        &Position,
        &mut Surface<TerrainMaterial>,
    )>() {
        // Don't generate the voxels and mesh for culled chunks or chunks that had
        // their mesh already generated
        if surface.culled || chunk.state == ChunkState::Generated {
            continue;
        }

        surface.visible = true;
        chunk.state = ChunkState::Generated;

        //chunk.state = ChunkState::Generated;
        log::trace!("terrain generation: voxels and mesh for chunk {}", chunk.coords);

        //terrain.counters.write(&[0, 0], 0).unwrap();
        let mesh = meshes.get(&surface.mesh);
        let indirect = mesh.indirect();
        let indirect = indirects.get_mut(indirect);

        // Reset the DrawIndexedIndirect element for this chunk
        indirect
            .write(
                &[DrawIndexedIndirect {
                    vertex_count: 0,
                    instance_count: 1,
                    base_index: 0,
                    vertex_offset: 0,
                    base_instance: 0,
                }],
                mesh.offset(),
            )
            .unwrap();

        // Reset the current counters
        terrain.counters.write(&[0; 2], 0).unwrap();
        terrain.offsets.write(&[u32::MAX, u32::MAX, u32::MAX], 0).unwrap();

        // Fetch the buffer used by this chunk from the terrain pool
        let output_vertices = vertices.get_mut(&terrain.shared_vertex_buffers[chunk.allocation]);
        let output_triangles = triangles.get_mut(&terrain.shared_triangle_buffers[chunk.allocation]);
        let ranges = &mut terrain.ranges[chunk.allocation];
        let temp_vertices = &mut terrain.temp_vertices;
        let temp_triangles = &mut terrain.temp_triangles;

        // Create a compute pass for both the voxel and mesh compute shaders
        let mut pass = ComputePass::begin(&graphics);

        // Create the voxel data and store it in the image
        let mut active = pass.bind_shader(&terrain.compute_voxels);

        // Set voxel noise parameters
        let factor =
            (terrain.size as f32 - 2.0) / (terrain.size as f32);
        active
            .set_push_constants(|x| {
                let offset = position.with_w(0.0f32) * factor;
                let offset = GpuPod::into_bytes(&offset);
                let time = time.elapsed().as_secs_f32();
                let time = GpuPod::into_bytes(&time);

                x.push(
                    offset,
                    0,
                    graphics::ModuleVisibility::Compute,
                )
                .unwrap();
                x.push(
                    time,
                    offset.len() as u32,
                    graphics::ModuleVisibility::Compute,
                )
                .unwrap();
            })
            .unwrap();

        // One global bind group for voxel generation
        active.set_bind_group(0, |set| {
            set.set_storage_texture(
                "densities",
                &mut terrain.densities,
            )
            .unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(terrain.dispatch));
        
        // Execute the vertex generation shader first
        let mut active = pass.bind_shader(&terrain.compute_vertices);

        active.set_bind_group(0, |set| {
            set.set_storage_texture(
                "densities",
                &mut terrain.densities,
            )
            .unwrap();
            set.set_storage_texture(
                "cached_indices",
                &mut terrain.cached_indices,
            )
            .unwrap();
            set.set_storage_buffer("counters", &mut terrain.counters)
                .unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("vertices", temp_vertices)
                .unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(terrain.dispatch));

        
        // Execute the quad generation shader second
        let mut active = pass.bind_shader(&terrain.compute_quads);
        active.set_bind_group(0, |set| {
            set.set_storage_texture(
                "cached_indices",
                &mut terrain.cached_indices,
            )            .unwrap();
            set.set_storage_texture(
                "densities",
                &mut terrain.densities,
            )
            .unwrap();
            set.set_storage_buffer("counters", &mut terrain.counters).unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("triangles", temp_triangles).unwrap();
            set.set_storage_buffer("indirect", indirect).unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(terrain.dispatch));

        // Run a compute shader that will iterate over the ranges and find a free one 
        let mut active = pass.bind_shader(&terrain.compute_find);
        active.set_bind_group(0, |set| {
            set.set_storage_buffer("ranges", ranges).unwrap();
            set.set_storage_buffer("offsets", &mut terrain.offsets).unwrap();
            set.set_storage_buffer("counters", &mut terrain.counters).unwrap();
        });

        /*
        // Copy the generated vertex and tri data to the permanent buffer 
        let mut active = pass.bind_shader(&terrain.compute_copy);
        active.set_bind_group(0, |set| {
            set.set_storage_buffer("temporary_vertices", temp_vertices)
                .unwrap();
            set.set_storage_buffer("temporary_triangles", temp_triangles)
                .unwrap();
            set.set_storage_buffer("old_counters", &mut terrain.old_counters)
                .unwrap();
            set.set_storage_buffer("new_counters", &mut terrain.counters)
                .unwrap();
            /*
            set.set_storage_buffer("current_counters", &mut terrain.current_counters)
                .unwrap();
            */
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("output_vertices", output_vertices)
                .unwrap();
            set.set_storage_buffer("output_triangles", output_triangles)
                .unwrap();
            set.set_storage_buffer("indirect", indirect).unwrap();
        });
        */

        active.dispatch(vek::Vec3::new(1, 1, 1));
        return;
    }
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}
