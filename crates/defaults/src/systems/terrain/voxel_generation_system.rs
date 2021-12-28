use super::VoxelGenerationSystem;
use core::global::callbacks::CallbackType;
use ecs::SystemData;
use others::callbacks::{MutCallback, NullCallback, OwnedCallback};
use rendering::{pipec, RenderTask, TextureShaderAccessType};
use terrain::{ChunkState, Voxel, VoxelData, ISOLINE, MAIN_CHUNK_SIZE};
ecs::impl_systemdata!(VoxelGenerationSystem);

// Get the first pending chunk, and tell the voxel generator to generate it's voxel data if it is allowed to
fn system_prefire(data: &mut SystemData<VoxelGenerationSystem>) {
    if !data.generating && data.pending_chunks.len() > 0 {
        // We can run the voxel generation logic
        let chunk_coords = data.pending_chunks.remove(0);
        println!("Started voxel generation for Chunk {}", chunk_coords.center);
        // Set the state
        data.generating = true;

        // First pass
        let mut group = rendering::ShaderUniformsGroup::new();
        group.update_shader_id(&data.compute);
        group.set_i3d("voxel_image", &data.voxel_texture, TextureShaderAccessType::WriteOnly);
        group.set_i3d("material_image", &data.material_texture, TextureShaderAccessType::WriteOnly);
        group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
        group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
        group.set_i32("node_size", chunk_coords.size as i32);
        group.set_i32("depth", chunk_coords.depth as i32);
        // Dispatch the compute shader, don't read back the data immediately
        let indices = (
            (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
            (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
            (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
        );
        let result = pipec::task(pipec::RenderTask::ComputeRun(data.compute, indices, group));
        // Callback data that we will pass
        let mut data = data.clone();
        result.with_callback(
            CallbackType::RenderingCommandExecution(NullCallback::new(move || {
                use std::sync::{Arc, Mutex};
                // This callback is executed when the compute shader finishes it's execution.
                // We can safely read back from the textures now

                // We will make a batch command, so we can send the two tasks at the same time
                let voxel_pixels: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
                let task1 = RenderTask::TextureFillArray(data.voxel_texture, std::mem::size_of::<f32>(), voxel_pixels.clone());
                let material_pixels: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
                let task2 = RenderTask::TextureFillArray(data.material_texture, std::mem::size_of::<u8>() * 2, material_pixels.clone());
                // Create the batch command
                pipec::task_batch(vec![pipec::task(task1), pipec::task(task2)]).with_callback(
                    CallbackType::RenderingCommandExecution(NullCallback::new(move || {
                        let i = std::time::Instant::now();
                        let voxel_pixels = Arc::try_unwrap(voxel_pixels).unwrap().into_inner().unwrap();
                        let material_pixels = Arc::try_unwrap(material_pixels).unwrap().into_inner().unwrap();
                        let voxel_pixels = pipec::convert_native::<f32>(voxel_pixels);
                        let material_pixels = pipec::convert_native_veclib::<veclib::Vector2<u8>, u8>(material_pixels);
                        // Keep track of the min and max values
                        let mut min = f32::MAX;
                        let mut max = f32::MIN;
                        // Turn the pixels into the data
                        let mut local_data: Box<[(f32, u8, u8)]> = vec![(0.0, 0, 0); (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)].into_boxed_slice();
                        let mut voxel_data: VoxelData = VoxelData {
                            voxels: vec![Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)].into_boxed_slice(),
                        };
                        for (i, density) in voxel_pixels.into_iter().enumerate() {
                            let material: veclib::Vector2<u8> = material_pixels[i];
                            // Keep the min and max
                            min = min.min(density);
                            max = max.max(density);
                            // Create the simplified voxel
                            let simplified_voxel_tuple = (density, material.x, material.y);
                            local_data[i] = simplified_voxel_tuple;
                        }
                        // If there is no surface, no need to waste time
                        let surface = min.signum() != max.signum();
                        if !surface {
                            data.result = Some((chunk_coords, None));
                            return;
                        };

                        // Flatten using the custom size of MAIN_CHUNK_SIZE+2
                        fn custom_flatten(x: usize, y: usize, z: usize) -> usize {
                            x + (y * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)) + (z * (MAIN_CHUNK_SIZE + 2))
                        }
                        // Calculate the voxel normal
                        for x in 0..(MAIN_CHUNK_SIZE + 1) {
                            for y in 0..(MAIN_CHUNK_SIZE + 1) {
                                for z in 0..(MAIN_CHUNK_SIZE + 1) {
                                    let i = custom_flatten(x, y, z);
                                    let v0 = local_data[i];
                                    // Calculate the normal using the difference between neigboring voxels
                                    let v1 = local_data[custom_flatten(x + 1, y, z)];
                                    let v2 = local_data[custom_flatten(x, y + 1, z)];
                                    let v3 = local_data[custom_flatten(x, y, z + 1)];
                                    // Normal
                                    let normal = veclib::Vector3::new(v1.0 as f32 - v0.0 as f32, v2.0 as f32 - v0.0 as f32, v3.0 as f32 - v0.0 as f32).normalized();
                                    let sv = local_data[i];
                                    let voxel = Voxel {
                                        density: sv.0,
                                        normal,
                                        material_id: sv.1,
                                    };
                                    voxel_data.voxels[terrain::utils::flatten((x, y, z))] = voxel;
                                }
                            }
                        }
                        // Tell the main system data that we finished the voxel generation for this specific chunk
                        data.result = Some((chunk_coords, Some(voxel_data)));
                    }))
                    .create(),
                );
            }))
            .create(),
        );
    }
}

// Check if the current Chunk has gotten it's voxel data generated
fn entity_update(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
    let chunk_coords = &core::global::ecs::component::<terrain::Chunk>(entity).unwrap().coords;

    // Check if we generated the voxel data for this chunk
    // The outer option is whether or not we have generated the Voxel Data
    // The inner option is whether or not we have a valid Voxel Data (Voxel Data with a valid surface)
    let voxel_data = if let Option::Some((generatin_chunk_coords, _)) = &data.result {
        // We are currently generating a chunk, but we are not sure if we are generating the correct one
        if generatin_chunk_coords == chunk_coords {
            // It is a match, return the voxel data
            let (_, taken) = data.result.take().unwrap();
            data.generating = false;
            Some(taken)
        } else {
            None
        }
    } else {
        None
    };

    if let Option::Some(voxel_data) = voxel_data {
        // We did generate the voxel data for this chunk, so update it
        core::global::ecs::entity_mut(
            entity.entity_id,
            CallbackType::LocalEntityMut(MutCallback::new(|entity| {
                // Update the chunk component
                let chunk = core::global::ecs::component_mut::<terrain::Chunk>(entity).unwrap();
                chunk.voxel_data = voxel_data;
                chunk.state = ChunkState::ValidVoxelData;
            }))
            .create(),
        );
    }
}

// When a chunk gets added, we tell the voxel generator to buffer the voxel generation for that chunk
fn entity_added(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
    let chunk_coords = core::global::ecs::component::<terrain::Chunk>(entity).unwrap().coords.clone();
    data.pending_chunks.push(chunk_coords);
    core::global::ecs::entity_mut(
        entity.entity_id,
        CallbackType::LocalEntityMut(MutCallback::new(|entity| {
            // Update the chunk state
            let chunk = core::global::ecs::component_mut::<terrain::Chunk>(entity).unwrap();
            chunk.state = ChunkState::PendingVoxelGeneration;
        }))
        .create(),
    );
}

// When a chunk gets removed, we tell the voxel generator to stop generating the chunk's voxel data, if it is
fn entity_removed(data: &mut SystemData<VoxelGenerationSystem>, entity: &ecs::Entity) {
    let chunk_coords = &core::global::ecs::component::<terrain::Chunk>(entity).unwrap().coords;
    let i = data.pending_chunks.iter().position(|x| x == chunk_coords);
    if let Option::Some(i) = i {
        data.pending_chunks.remove(i);
    }
}

// Create the default system
pub fn system(interpreter_string: String) {
    // Create the system data
    core::global::ecs::add_system(VoxelGenerationSystem::new(interpreter_string), || {
        // Create a system
        let mut system = ecs::System::new();
        // Link some components to the system
        system.link::<crate::components::Transform>();
        system.link::<terrain::Chunk>();
        // And link the events
        system.event(ecs::SystemEventType::SystemPrefire(system_prefire));
        system.event(ecs::SystemEventType::EntityUpdate(entity_update));
        system.event(ecs::SystemEventType::EntityAdded(entity_added));
        system.event(ecs::SystemEventType::EntityRemoved(entity_removed));
        // Return the newly made system
        system
    });
}
