use assets::Assets;
use ecs::Scene;
use graphics::{ComputePass, Graphics, DrawIndexedIndirectBuffer, DrawIndexedIndirect, Texture, GpuPod};
use rendering::{Mesh, Pipelines, Surface, Basic, Renderer};
use utils::{Storage, Time};
use world::{System, World};

use crate::{VoxelGenerator, MeshGenerator, Terrain};

// Called at the start of the app to add the resources
fn init(world: &mut World) {
    world.insert::<Storage<Terrain>>(Default::default());
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();
    let mut indirect = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut materials = world.get_mut::<Storage<Terrain>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Global chunk resolution
    let size = 64;

    // Create the compute generators
    let voxel = VoxelGenerator::new(&graphics, &assets, size);
    let mesh = MeshGenerator::new(&graphics, &assets, &mut indirect, &mut meshes, size);

    // Create a basic terrain material
    let id = pipelines.register::<Terrain>(&graphics, &assets).unwrap();
    let material = materials.insert(Terrain {
        bumpiness: 1.0,
        roughness: 0.9,
        metallic: 0.0,
        ambient_occlusion: 0.0,
    });

    // Add the debug mesh into the world
    let surface = Surface::indirect(mesh.mesh.clone(), material, mesh.indirect.clone(), id);
    scene.insert((surface, Renderer::default()));

    // Add the resources to the world
    drop(graphics);
    drop(assets);
    drop(indirect);
    drop(meshes);
    drop(materials);
    drop(scene);
    drop(pipelines);
    world.insert(voxel);
    world.insert(mesh);
}

// Called each frame before rendering to generate the required voxel data and mesh data for each chunk
fn update(world: &mut World) {
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
}

// Responsible for terrain generation and rendering
pub fn system(system: &mut System) {
    system.insert_init(init);
    system
        .insert_update(update)
        .before(rendering::systems::rendering::system);
}
