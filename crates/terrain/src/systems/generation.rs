use ecs::{Scene, Position};
use graphics::{Graphics, DrawIndexedIndirectBuffer, DrawIndexedIndirect, ComputePass, GpuPod};
use rendering::{Mesh, Surface, attributes};
use utils::{Time, Storage};
use world::{System, World};

use crate::{Terrain, Chunk, ChunkState, TerrainMaterial};

// Look in the world for any chunks that need their mesh generated and generate it
fn update(world: &mut World) { 
    let graphics = world.get::<Graphics>().unwrap();
    let time = world.get::<Time>().unwrap();
    let mut _terrain = world.get_mut::<Terrain>().unwrap();
    let terrain = &mut *_terrain;
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut indirects = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();

    // Iterate over the chunks that we need to generate
    for (chunk, position, surface) in scene.query_mut::<(&mut Chunk, &Position, &Surface<TerrainMaterial>)>() {
        if let ChunkState::Generated = chunk.state {
            return;
        }
        chunk.state = ChunkState::Generated;

        terrain.counters.write(&[0, 0], 0).unwrap();
        
        let indirect = indirects.get_mut(surface.indirect.as_ref().unwrap());
        let mesh = meshes.get_mut(&surface.mesh);
        let (mut triangles, vertices) = mesh.both_mut();
        let triangles = triangles.buffer_mut();
        let mut positions = vertices.attribute_mut::<attributes::Position>().unwrap();
        let mut normals = vertices.attribute_mut::<attributes::Normal>().unwrap();

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
        let mut active = pass.bind_shader(&terrain.compute_voxels);

        // Set voxel noise parameters
        active.set_push_constants(|x| {
            let position = GpuPod::into_bytes(&**position);
            let time = time.elapsed().as_secs_f32();
            let time = GpuPod::into_bytes(&time);

            x.push(position, 0, graphics::ModuleVisibility::Compute).unwrap();
            x.push(time, position.len() as u32, graphics::ModuleVisibility::Compute).unwrap();
        }).unwrap();

        // One global bind group for voxel generation
        active.set_bind_group(0, |set| {
            set.set_storage_texture("densities", &mut terrain.densities)
                .unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(terrain.dispatch));

        // Execute the vertex generation shader first
        let mut active = pass.bind_shader(&terrain.compute_vertices);

        active.set_bind_group(0, |set| {
            set.set_storage_texture("densities", &mut terrain.densities).unwrap();
            set.set_storage_texture("cached_indices", &mut terrain.cached_indices).unwrap();
            set.set_storage_buffer("counters", &mut terrain.counters).unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("vertices", &mut positions).unwrap();
            set.set_storage_buffer("normals", &mut normals).unwrap();
        });
        active.dispatch(vek::Vec3::broadcast(terrain.dispatch)); 

        // Execute the quad generation shader second
        let mut active = pass.bind_shader(&terrain.compute_quads);
        active.set_bind_group(0, |set| {
            set.set_storage_texture("densities", &mut terrain.densities).unwrap();
            set.set_storage_texture("cached_indices", &mut terrain.cached_indices).unwrap();
        });
        active.set_bind_group(1, |set| {
            set.set_storage_buffer("triangles", triangles).unwrap();
            set.set_storage_buffer("indirect", indirect).unwrap();
        });

        active.dispatch(vek::Vec3::broadcast(terrain.dispatch));  
    }
}

// Generates the voxels and appropriate mesh for each of the visible chunks 
pub fn system(system: &mut System) {
    system.insert_update(update)
        .before(rendering::systems::rendering::system);
}
