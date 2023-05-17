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
        &mut terrain.manager,
        &mut terrain.memory,
        &terrain.culler,
    );

    // Back-propagate the count if needed
    let mut scene = world.get_mut::<Scene>().unwrap();
    for node in manager.octree.nodes() {
        // We only care about parent nodes
        if node.children().is_none() {
            continue;
        }

        // Check if we should backpropagate to the parent node of this parent node
        let mut backpropagate = false;
        if let Some((count, e)) = manager.counting.get_mut(&node.center()) {
            if *count == 8 {
                backpropagate = true;
            }
        }

        // If we have a parent node, update it
        if backpropagate {
            let this = manager.counting.get(&node.center()).unwrap().clone();
            let parent = node.parent().map(|x| manager.octree.nodes()[x].center());

            if let Some((count, entities)) = parent.map(|x| manager.counting.get_mut(&x)).flatten() {
                // The parent got a new child done generating (this)
                *count += 1;
                entities.extend(this.1);
            } else {
                // If we don't have a parent it means we are either the root node or this is the last parent node in the sub tree that we must generate
                for x in this.1.iter() {
                    let entry = scene.entry(*x).unwrap();
                    let chunk = entry.get::<Chunk>().unwrap(); 
                    memory.visibility_bitsets[chunk.allocation].set(chunk.local_index);   
                }
            }
            
            // Delete "this" node since it was already handled
            manager.counting.remove(&node.center()).unwrap();
        }
    }

    // I AM OOPDATINGG AAAAAAAAAAAA
    let time = world.get::<Time>().unwrap();
    let allocation = (time.frame_count() as usize) % 2;
    let chunks = memory.visibility_bitsets[allocation].chunks();
    memory.visibility_buffers[allocation].write(chunks, 0).unwrap();

    // Hide the parent nodes that had their children generated
    let query = scene.query_mut::<&mut Chunk>().into_iter()
        .filter_map(|c| c.node.map(|x| (x, c)))
        .filter(|(_, c)| c.state == ChunkState::Removed);
    for (node, chunk) in query {
        // If we are a parent node then wait till we are done generating our children to remove us
        if !manager.counting.contains_key(&node.center()) {
            memory.visibility_bitsets[chunk.allocation].remove(chunk.local_index);
            chunk.state = ChunkState::Free;
        }
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
        .after(crate::systems::readback::system)
        .before(rendering::systems::rendering::system);
}
