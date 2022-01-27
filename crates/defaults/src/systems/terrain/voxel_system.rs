use main::{
    core::{Context, WriteContext},
    ecs::{component::{ComponentQuery, ComponentID}, entity::EntityID}, terrain::{ChunkCoords, MAIN_CHUNK_SIZE, VoxelData, Voxel, ISOLINE}, rendering::{advanced::{compute::ComputeShaderExecutionSettings, atomic::{AtomicGroup, AtomicGroupRead}}, basics::{uniforms::ShaderUniformsGroup, texture::{TextureAccessType, TextureReadBytes}, transfer::Transferable}, pipeline::{pipec, Pipeline}, object::PipelineTrackedTask},
};

use crate::globals::TerrainGenerationData;

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
    // Create the compute shader execution settings and execute the compute shader
    let compute = terrain.compute_shader;
    const AXIS: u16 = 1;    
    // Set the uniforms for the compute shader as well
    let mut group = ShaderUniformsGroup::new();
    group.set_image("density_image", terrain.density_texture, TextureAccessType::WRITE);
    group.set_image("material_image", terrain.material_texture, TextureAccessType::WRITE);
    group.set_f32("isoline", ISOLINE);

    // Chunk specific uniforms
    let chunk_coords = chunk.coords;
    group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
    group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
    group.set_i32("node_size", chunk_coords.size as i32);
    group.set_i32("depth", chunk_coords.depth as i32);

    // Create this for the next step
    let read_densities = TextureReadBytes::default();
    let read_materials = TextureReadBytes::default();
    let read_counters = AtomicGroupRead::default();
    let read_densities_transfer = read_densities.transfer();
    let read_materials_transfer = read_materials.transfer();
    let read_counters_transfer = read_counters.transfer();
    // Set the atomic counter
    group.set_atomic_group("_", terrain.counters, 2);

    // Now we can execute the compute shader and the read bytes command
    let execution_settings = ComputeShaderExecutionSettings::new(compute, (AXIS, AXIS, AXIS)).set_uniforms(group);
    let execution = pipec::tracked_task(PipelineTrackedTask::RunComputeShader(compute, execution_settings), None, pipeline);
    
    let read_densities_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.density_texture, read_densities_transfer), Some(execution), pipeline);
    let read_materials_tracked_id = pipec::tracked_task(PipelineTrackedTask::TextureReadBytes(terrain.material_texture, read_materials_transfer), Some(execution), pipeline);
    let read_counters_tracked_id = pipec::tracked_task(PipelineTrackedTask::AtomicGroupRead(terrain.counters, read_counters_transfer), Some(execution), pipeline);

    // Combine the tasks to make a finalizer one
    let main = pipec::tracked_finalizer(vec![execution, read_densities_tracked_id, read_materials_tracked_id, read_counters_tracked_id], pipeline).unwrap();
    terrain.generating = Some(TerrainGenerationData {
        main_id: main,
        chunk_id: id,
        texture_reads: (read_densities, read_materials),
        atomic_read: read_counters,
    });
    println!("Dispatched voxel generation!");
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
    println!("Finished voxel data generation!");
    // Load the read containers that we passed to the pipeline
    let data = terrain.generating.as_ref().unwrap();

    // Load the actual voxels now
    //let voxel_pixels = read_densities.fill_vec::<f32>().unwrap();
    //let material_pixels = read_materials.fill_vec::<veclib::Vector2<u8>>().unwrap();
    
    // Keep track of the min and max values
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    // Create the voxel data on the heap since it's going to be pretty big
    let mut local_data: Box<[(f32, u8)]> = vec![(0.0, 0); (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)].into_boxed_slice();
    let mut voxel_data: VoxelData = VoxelData(vec![Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)].into_boxed_slice());
    let positive = data.atomic_read.get(0).unwrap();
    let negative = data.atomic_read.get(1).unwrap();

    println!("{} {}", positive, negative);

    /*
    // 
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
                data.results.insert(chunk_coords, Some(None));
                /*
                println!(
                    "Finished voxel generation for Chunk {}, took {}ms (Async {}ms). [NO VALID SURFACE FOUND]",
                    chunk_coords.center,
                    i.elapsed().as_millis(),
                    i1
                );
                */
                // We finished generating data on this compute shader
                let x = data.computes.get_mut(compute_index).unwrap();
                x.1 = false;
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
            /*
            println!(
                "Finished voxel generation for Chunk {}, took {}ms (Async {}ms)",
                chunk_coords.center,
                i.elapsed().as_millis(),
                i1
            );
            */
            let x = data.computes.get_mut(compute_index).unwrap();
            x.1 = false;
            // Tell the main system data that we finished the voxel generation for this specific chunk
            data.results.insert(chunk_coords, Some(Some(voxel_data)));
        }))
        .create(),
    );
    // Keep track of the min and max values
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    // Turn the pixels into the data
    let mut voxel_data: VoxelData = VoxelData {
        voxels: vec![Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)].into_boxed_slice(),
    };
    // If there is no surface, no need to waste time
    /*
    let surface = min.signum() != max.signum();
    if !surface {
        data.results.insert(chunk_coords, Some(None));
        /*
        println!(
            "Finished voxel generation for Chunk {}, took {}ms (Async {}ms). [NO VALID SURFACE FOUND]",
            chunk_coords.center,
            i.elapsed().as_millis(),
            i1
        );
        */
        // We finished generating data on this compute shader
        let x = data.computes.get_mut(compute_index).unwrap();
        x.1 = false;
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
                // Normal
                let voxel = Voxel {
                    density: y as f32 - 10.0,
                    normal: veclib::Vector3::default(),
                    material_id: 0,
                };
                voxel_data.voxels[terrain::utils::flatten((x, y, z))] = voxel;
            }
        }
    }
    /*
    println!(
        "Finished voxel generation for Chunk {}, took {}ms (Async {}ms)",
        chunk_coords.center,
        i.elapsed().as_millis(),
        i1
    );
    */
    // Tell the main system data that we finished the voxel generation for this specific chunk
    */
    */
}


// The voxel systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read().unwrap();
    
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk in the terrain, we must create it's respective voxel data, if possible
        if terrain.generating.is_none() {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that we come across that needs it
            query.update_all_breakable(|components| {
                // We break out at the first chunk if we start generating it's voxel data
                let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                if chunk.voxel_data.is_none() {
                    // We must start generating the voxel data for this chunk
                    start_generation(&mut *terrain, &*pipeline, &mut *chunk, components.get_entity_id().unwrap());
                }
                None
            })
        } else {
            // We must check if we have finished generating or not
            if pipec::has_task_executed(terrain.generating.as_ref().unwrap().main_id, &*pipeline).unwrap() {
                // We will now update the chunk data to store our new voxel data
                let id = terrain.generating.as_ref().unwrap().chunk_id;
                query.update(id, |components| {
                    // Get our chunk and set it's new data
                    let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                    finish_generation(&mut *terrain, &*pipeline, &mut *chunk);
                });
            }
        }
    }
}
// Create a voxel system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
