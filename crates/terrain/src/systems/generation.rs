use ecs::{Position, Scene};
use graphics::{
    ComputePass, DrawIndexedIndirect, DrawIndexedIndirectBuffer,
    GpuPod, Graphics, TriangleBuffer, Buffer, ActiveComputePass, XYZW, VertexBuffer, ComputeShader, Vertex,
};
use rendering::{
    attributes, AttributeBuffer, IndirectMesh, Mesh, Surface,
};
use utils::{Storage, Time};
use world::{System, World};
use crate::{Chunk, ChunkState, Terrain, TerrainMaterial, Densities, CachedIndices};

// Utility struct that contains all data mutably borrowed from terrain
struct BorrowedTerrainData<'a> {
    pub(crate) compute_quads: &'a ComputeShader,
    pub(crate) densities: &'a mut Densities,

    pub(crate) shared_vertex_buffers:
        &'a mut AttributeBuffer<attributes::Position>,
    pub(crate) shared_triangle_buffers:
        &'a mut TriangleBuffer<u32>,

    pub(crate) temp_vertices: &'a mut Buffer<<XYZW<f32> as Vertex>::Storage>,
    pub(crate) temp_triangles: &'a mut Buffer<[u32; 3]>,

    pub(crate) compute_vertices: &'a ComputeShader,
    pub(crate) compute_voxels: &'a ComputeShader,
    pub(crate) cached_indices: &'a CachedIndices,

    pub(crate) compute_find: &'a ComputeShader,

    pub(crate) offsets: &'a mut Buffer<u32>,

    pub(crate) counters: &'a mut Buffer<u32>,

    pub(crate) sub_allocation_chunk_indices: &'a mut Buffer<u32>,

    pub(crate) compute_copy: &'a ComputeShader,

    pub(crate) dispatch: u32,

    pub(crate) size: u32,
    pub(crate) allocations: usize,
    pub(crate) sub_allocations: usize,
    pub(crate) chunks_per_allocation: usize,
    pub(crate) chunk_render_distance: u32,
}

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
    let mut vertices = world
        .get_mut::<Storage<AttributeBuffer<attributes::Position>>>()
        .unwrap();
    let mut triangles =
        world.get_mut::<Storage<TriangleBuffer<u32>>>().unwrap();
    let mut meshes =
        world.get_mut::<Storage<IndirectMesh>>().unwrap();

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
        log::trace!(
            "terrain generation: voxels and mesh for chunk {}",
            chunk.coords
        );

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
        terrain.offsets.write(&[u32::MAX, u32::MAX], 0).unwrap();

        // Create big struct that will contain most of the shit
        let util = BorrowedTerrainData {
            compute_quads: todo!(),
            densities: todo!(),
            shared_vertex_buffers: todo!(),
            shared_triangle_buffers: todo!(),
            temp_vertices: todo!(),
            temp_triangles: todo!(),
            compute_vertices: todo!(),
            compute_voxels: todo!(),
            cached_indices: todo!(),
            compute_find: todo!(),
            offsets: todo!(),
            counters: todo!(),
            sub_allocation_chunk_indices: &mut terrain.sub_allocation_chunk_indices[chunk.allocation],
            compute_copy: &terrain.compute_copy,
            dispatch: terrain.dispatch,
            
            size: terrain.size,
            allocations: terrain.allocations,
            sub_allocations: terrain.sub_allocations,
            chunks_per_allocation: terrain.chunks_per_allocation,
            chunk_render_distance: terrain.chunk_render_distance,
        };

        // Fetch the buffer used by this chunk from the terrain pool
        let output_vertices = vertices.get_mut(
            &terrain.shared_vertex_buffers[chunk.allocation],
        );
        let output_triangles = triangles.get_mut(
            &terrain.shared_triangle_buffers[chunk.allocation],
        );
        let sub_allocation_chunk_indices = ;
        let temp_vertices = &mut terrain.temp_vertices;
        let temp_triangles = &mut terrain.temp_triangles;

        // Create a compute pass for both the voxel and mesh compute shaders
        let mut pass = ComputePass::begin(&graphics);


        // 3 compute shaders that will generate the data and write it to
        // temp_vertices and temp_triangles TEMPORARILY
        generate_voxels(&mut pass, terrain, position, chunk);
        generate_vertices(&mut pass, terrain, temp_vertices);
        generate_quads(
            &mut pass,
            terrain,
            temp_triangles,
            indirect,
            mesh,
        );

        // Compute shader that will try to find a free memory location within the current allocation
        find_free_memory(
            &mut pass,
            terrain,
            sub_allocation_chunk_indices,
            chunk,
        );

        // Shader that will copy the temporary data into the allocation for permanent storage
        copy_to_free_memory(
            pass,
            terrain,
            temp_vertices,
            temp_triangles,
            output_vertices,
            output_triangles,
            indirect,
            mesh,
        );

        return;
    }
}

// Create an active compute shader pass that will generate the vertices
fn generate_vertices(
    pass: &mut ActiveComputePass,
    borrowed: &mut BorrowedTerrainData,
) {
    // Execute the vertex generation shader first
    let mut active = pass.bind_shader(&terrain.compute_vertices);

    active.set_bind_group(0, |set| {
        set.set_storage_texture("densities", &mut terrain.densities)
            .unwrap();
        set.set_storage_texture(
            "cached_indices",
            &mut terrain.cached_indices,
        )
        .unwrap();
        set.set_storage_buffer("counters", &mut terrain.counters, ..)
            .unwrap();
    });
    active.set_bind_group(1, |set| {
        set.set_storage_buffer("vertices", temp_vertices, ..)
            .unwrap();
    });
    active.dispatch(vek::Vec3::broadcast(terrain.dispatch));
}

// Create an active compute shader pass that will generate the quads
fn generate_quads(
    pass: &mut ActiveComputePass,
    borrowed: &mut BorrowedTerrainData,
    indirect: &mut DrawIndexedIndirectBuffer,
    mesh: &Mesh<rendering::Indirect>,
) {
    // Execute the quad generation shader second
    let mut active = pass.bind_shader(&terrain.compute_quads);
    active.set_bind_group(0, |set| {
        set.set_storage_texture(
            "cached_indices",
            &mut terrain.cached_indices,
        )
        .unwrap();
        set.set_storage_texture("densities", &mut terrain.densities)
            .unwrap();
        set.set_storage_buffer("counters", &mut terrain.counters, ..)
            .unwrap();
    });
    active.set_bind_group(1, |set| {
        set.set_storage_buffer("triangles", temp_triangles, ..)
            .unwrap();
        set.set_storage_buffer("indirect", indirect, ..).unwrap();
    });

    active
        .set_push_constants(|x| {
            let index = mesh.offset() as u32;
            let index = GpuPod::into_bytes(&index);
            x.push(index, 0, graphics::ModuleVisibility::Compute)
                .unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::broadcast(terrain.dispatch));
}

// Create an active compute shader pass that will find some free memory that we can copy to
fn find_free_memory(
    pass: &mut ActiveComputePass,
    borrowed: &mut BorrowedTerrainData,
) {
    // Run a compute shader that will iterate over the ranges and find a free one
    let mut active = pass.bind_shader(&compute_find);
    active.set_bind_group(0, |set| {
        set.set_storage_buffer(
            "indices",
            sub_allocation_chunk_indices,
            ..,
        )
        .unwrap();
        set.set_storage_buffer("offsets", offsets, ..)
            .unwrap();
        set.set_storage_buffer("counters",counters, ..)
            .unwrap();
    });

    active
        .set_push_constants(|x| {
            let index = chunk.index as u32
                % chunks_per_allocation as u32;
            let index = GpuPod::into_bytes(&index);
            x.push(index, 0, graphics::ModuleVisibility::Compute)
                .unwrap();
        })
        .unwrap();

    //let dispatch = (terrain.sub_allocations as f32 / 32 as f32).ceil() as u32;
    active.dispatch(vek::Vec3::new(1, 1, 1));
}

// Create an active compute shader pass that will copy the memory into the free memory
fn copy_to_free_memory(
    mut pass: ActiveComputePass,
    terrain: &mut Terrain,
    borrowed: &mut BorrowedTerrainData,
    indirect: &mut DrawIndexedIndirectBuffer,
    mesh: &Mesh<rendering::Indirect>,
) {
    // Copy the generated vertex and tri data to the permanent buffer
    let mut active = pass.bind_shader(&terrain.compute_copy);
    active.set_bind_group(0, |set| {
        set.set_storage_buffer(
            "temporary_vertices",
            temp_vertices,
            ..,
        )
        .unwrap();
        set.set_storage_buffer(
            "temporary_triangles",
            temp_triangles,
            ..,
        )
        .unwrap();
        set.set_storage_buffer("counters", &mut terrain.counters, ..)
            .unwrap();
        set.set_storage_buffer("offsets", &mut terrain.offsets, ..)
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
            x.push(index, 0, graphics::ModuleVisibility::Compute)
                .unwrap();
        })
        .unwrap();
    active.dispatch(vek::Vec3::new(2048, 1, 1));
}

// Create an active compute shader pass that will generate the voxels and voxel colors
fn generate_voxels(
    pass: &mut ActiveComputePass,
    borrowed: &mut BorrowedTerrainData,
    position: &Position,
    chunk: &mut Chunk,
) {
    // Create the voxel data and store it in the image
    let mut active = pass.bind_shader(&terrain.compute_voxels);

    // Set voxel noise parameters
    let factor = (terrain.size as f32 - 2.0) / (terrain.size as f32);
    active
        .set_push_constants(|x| {
            let offset = position.with_w(0.0f32) * factor;
            let offset = GpuPod::into_bytes(&offset);
            let packed =
                vek::Vec2::new(chunk.index, chunk.allocation)
                    .as_::<u32>();
            let time = GpuPod::into_bytes(&packed);

            x.push(offset, 0, graphics::ModuleVisibility::Compute)
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
        set.set_storage_texture("densities", &mut terrain.densities)
            .unwrap();
    });
    active.dispatch(vek::Vec3::broadcast(terrain.dispatch));
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}
