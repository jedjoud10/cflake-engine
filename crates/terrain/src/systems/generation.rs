use world::{System, World};

// Look in the world for any chunks that need their mesh generated and generate it
fn update(world: &mut World) {  
    /*
    let graphics = world.get::<Graphics>().unwrap();
    let time = world.get::<Time>().unwrap();
    let mut _voxel_generator = world.get_mut::<VoxelGenerator>().unwrap();
    let voxel_generator = &mut *_voxel_generator;
    let mut _mesh_generator = world.get_mut::<MeshGenerator>().unwrap();
    let mesh_generator = &mut *_mesh_generator;
    let mut indirect = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let indirect = indirect.get_mut(&mesh_generator.indirect);

    // Get the mesh that we will write to
    let mesh = meshes.get_mut(&mesh_generator.mesh);
    let (mut _triangles, mut _vertices) = mesh.both_mut();
    let mut vertices = _vertices.attribute_mut::<rendering::attributes::Position>().unwrap();
    let mut normals = _vertices.attribute_mut::<rendering::attributes::Normal>().unwrap();
    let triangles = _triangles.buffer_mut();

    mesh_generator.counters.write(&[0, 0], 0).unwrap();
    indirect.write(&[DrawIndexedIndirect {
        vertex_count: 0,
        instance_count: 1,
        base_index: 0,
        vertex_offset: 0,
        base_instance: 0,
    }], 0).unwrap();

    // Create a compute pass for both the voxel and mesh compute shaders
    let mut pass = ComputePass::begin(&graphics);

    // Create the voxel data and store it in the image
    let mut active = pass.bind_shader(&voxel_generator.shader);

    // Set voxel noise parameters
    active.set_push_constants(|x| {
        x.push(GpuPod::into_bytes(&time.elapsed().as_secs_f32()), 0, graphics::ModuleVisibility::Compute).unwrap()
    }).unwrap();

    // One glboal bind group for voxel generation
    active.set_bind_group(0, |set| {
        set.set_storage_texture("densities", &mut voxel_generator.densities)
            .unwrap();
    });
    active.dispatch(vek::Vec3::broadcast(voxel_generator.dispatch));

    // Execute the vertex generation shader first
    let mut active = pass.bind_shader(&mesh_generator.compute_vertices);

    active.set_bind_group(0, |set| {
        set.set_storage_texture("densities", &mut voxel_generator.densities).unwrap();
        set.set_storage_texture("cached_indices", &mut mesh_generator.cached_indices).unwrap();
        set.set_storage_buffer("counters", &mut mesh_generator.counters).unwrap();
    });
    active.set_bind_group(1, |set| {
        set.set_storage_buffer("vertices", &mut vertices).unwrap();
        set.set_storage_buffer("normals", &mut normals).unwrap();
    });
    active.dispatch(vek::Vec3::broadcast(mesh_generator.dispatch)); 

    // Execute the quad generation shader second
    let mut active = pass.bind_shader(&mesh_generator.compute_quads);
    active.set_bind_group(0, |set| {
        set.set_storage_texture("densities", &mut voxel_generator.densities).unwrap();
        set.set_storage_texture("cached_indices", &mut mesh_generator.cached_indices).unwrap();
    });
    active.set_bind_group(1, |set| {
        set.set_storage_buffer("triangles", triangles).unwrap();
        set.set_storage_buffer("indirect", indirect).unwrap();
    });

    active.dispatch(vek::Vec3::broadcast(mesh_generator.dispatch));  
    */ 
}

// Generates the voxels and appropriate mesh for each of the visible chunks 
pub fn generation(system: &mut System) {
}
