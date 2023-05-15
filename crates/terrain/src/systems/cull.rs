use ecs::Scene;
use graphics::{ComputePass, Graphics, ActivePipeline, GpuPod, DrawCountIndirectBuffer, DrawIndexedIndirectBuffer};
use rendering::Surface;
use utils::{Time, Storage};
use world::{System, World};

use crate::{Terrain, Chunk, TerrainMaterial, ChunkState};

// This will iterate over the generated indexed indirect buffers and cull the chunks that are not visible
// The culling will be based on frustum culling and simple visiblity (flag) culling
fn update(world: &mut World) {
    let Ok(mut _terrain) = world.get_mut::<Terrain>() else {
        return;
    };
    
    // Decompose the terrain into its subresources
    let terrain = &mut *_terrain;
    let (manager, memory, culler) = (
        &terrain.manager,
        &mut terrain.memory,
        &terrain.culler,
    );
    
    let time = world.get::<Time>().unwrap();
    let allocation = (time.frame_count() as usize) % 2;

    // I AM OOPDATINGG AAAAAAAAAAAA
    let chunks = memory.visibility_bitsets[allocation].chunks();
    memory.visibility_buffers[allocation].write(chunks, 0).unwrap();

    // 2 ways a node can be generated

    // Hide the parent nodes that had their children generated
    let mut scene = world.get_mut::<Scene>().unwrap();
    let query = scene.query_mut::<&mut Chunk>().into_iter()
        .filter_map(|c| c.node.map(|x| (x, c)))
        .filter(|(_, c)| c.state == ChunkState::Removed);
    for (node, chunk) in query {
        chunk.state = ChunkState::Free;
        /*
        // Node splits into 8 children chunks and those get generated (leaf node turns into parent, +8, -1)
        if let Some((generated, target, _)) = manager.children_count.get(&node.center()).cloned() {
            if generated.len() == target {
                // Hide the parent node
                memory.visibility_bitsets[chunk.allocation].remove(chunk.local_index);

                // Show the children nodes
                for i in generated {
                    memory.visibility_bitsets[i.allocation].set(i.local_index);
                }

                chunk.state = ChunkState::Free;
            }               
        } else {
            // 8 children chunks merge into a big node (parent node turns into leaf, +1, -7)
            //memory.visibility_bitsets[chunk.allocation].remove(chunk.local_index);
            //chunk.state = ChunkState::Free;
        }
        */
    }
    
    // Create compute pass that will cull the indexed indirect buffers
    let mut draw_count_indirect_buffers = world.get_mut::<Storage<DrawCountIndirectBuffer>>().unwrap();
    let mut indexed_indirect_buffers = world.get_mut::<Storage<DrawIndexedIndirectBuffer>>().unwrap();
    let culled_count_buffer = draw_count_indirect_buffers.get_mut(&memory.culled_count_buffer);
    let output_indirect = indexed_indirect_buffers.get_mut(&memory.culled_indexed_indirect_buffers[allocation]);

    let graphics = world.get::<Graphics>().unwrap();
    let mut pass = ComputePass::begin(&graphics);
    let mut active = pass.bind_shader(&culler.compute_cull);

    culled_count_buffer.write(&[0], allocation).unwrap();

    // Set the bind resources for bind group 0
    active.set_bind_group(0, |group| {
        group.set_storage_buffer("visibility", &memory.visibility_buffers[allocation], ..).unwrap();
        group.set_storage_buffer_mut("count", culled_count_buffer, ..).unwrap();
    }).unwrap();
    
    // Set the bind resources for bind group 1
    active.set_bind_group(1, |group| {
        group.set_storage_buffer("input_position_scale", &memory.generated_position_scaling_buffers[allocation], ..).unwrap();
        group.set_storage_buffer("input_indirect", &memory.generated_indexed_indirect_buffers[allocation], ..).unwrap();

        group.set_storage_buffer_mut("output_position_scale", &mut memory.culled_position_scaling_buffers[allocation], ..).unwrap();
        group.set_storage_buffer_mut("output_indirect", output_indirect, ..).unwrap();
    }).unwrap();
    
    // Set allocation push constants
    active.set_push_constants(|pc| {
        let allocation = allocation as u32;
        let bytes = GpuPod::into_bytes(&allocation);
        pc.push(bytes, 0).unwrap();

        let chunks_per_allocation = manager.chunks_per_allocation as u32;
        pc.push(GpuPod::into_bytes(&chunks_per_allocation), bytes.len() as u32).unwrap();
    }).unwrap();

    // TODO: Run the culling compute shader with the data from the camera
    active.dispatch(vek::Vec3::new(128, 1, 1)).unwrap();

    drop(active);
    drop(pass);

    graphics.submit(false);
}

// Generates the voxels and appropriate mesh for each of the visible chunks
pub fn system(system: &mut System) {
    system
        .insert_update(update)
        .after(crate::systems::manager::system)
        .after(crate::systems::generation::system)
        .before(rendering::systems::rendering::system);
}
