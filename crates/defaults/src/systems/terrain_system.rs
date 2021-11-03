use std::time::Instant;

use debug::DefaultDebugRendererType;
use terrain::{BoundChecker, ChunkCoords, TerrainStats};

use crate::components;
use components::Chunk;
use ecs::*;
use math::octrees::*;
use rendering::*;
use systems::*;
use world_data::WorldData;

pub struct CustomData {
    pub lod_factor: f32,
    pub nodes: Vec<OctreeNode>,
    pub terrain_gen: bool,
    pub terrain_stats: TerrainStats,
}
crate::impl_custom_system_data!(CustomData);

// Events
fn system_prefire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();
    // Println the terrain stats
    if data.debug.console.listen_command("stat-terrain").is_some() {
        println!("{:?}", system.terrain_stats);
    }
}
fn entity_update(system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Timing
    let t = Instant::now();
    // Get the camera location
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let camera_transform = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap();
    let system = system_data.cast_mut::<CustomData>().unwrap();
    // Get the camera transform values
    let camera_location = camera_transform.position;
    let camera_forward_vector = camera_transform.get_forward_vector();
    let camera_velocity = camera_entity.get_component::<components::Physics>(data.component_manager).unwrap().object.linear.velocity;

    // Get the terrain data
    let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    let bound_materials = td.bound_materials.clone();
    let octree_size = td.octree.internal_octree.size;

    // Generate the octree each frame and generate / delete the chunks
    if data.debug.console.listen_command("toggle-terrain-gen").is_some() {
        system.terrain_gen = !system.terrain_gen;
    }
    if td.chunk_manager.octree_update_valid() && system.terrain_gen {
        match td.octree.generate_incremental_octree(&camera_location, &camera_velocity, system.lod_factor) {
            Some((mut added, removed, total)) => {
                system.nodes = total;
                // Filter first
                added.retain(|node| BoundChecker::bound_check(&node) && node.children_indices.is_none());
                system.terrain_stats.max_chunks_generated = system.terrain_stats.max_chunks_generated.max(added.len());
                // Turn all the newly added nodes into chunks and instantiate them into the world
                for octree_node in added {
                    // Add the chunk in the chunk manager
                    td.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                }
                // Delete all the removed octree nodes from the world
                system.terrain_stats.max_chunks_deleted = system.terrain_stats.max_chunks_deleted.max(removed.len());
                for octree_node in removed {
                    let chunk_coords = ChunkCoords::new(&octree_node);
                    // Remove the chunk from the chunk manager
                    match td.chunk_manager.remove_chunk(&chunk_coords) {
                        Some(_) => {
                            // Get the entity id
                            td.chunk_manager.remove_chunk_entity(&chunk_coords);
                        }
                        None => {}
                    }
                }
            }
            None => { /* Nothing happened */ }
        }
        td.chunk_manager.update_camera_view(camera_location, camera_forward_vector);
    }

    // Update the chunk manager
    let (added_chunks, removed_chunks) = td.chunk_manager.update(&mut td.voxel_generator, data.time_manager.frame_count);
    let mut added_chunk_entities_ids: Vec<(usize, ChunkCoords)> = Vec::new();

    // Add the entities to the entity manager
    for (coords, tmodel) in added_chunks {
        // Add the entity
        let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
        let mut entity = Entity::new(name.as_str());

        // Create the chunk component
        let chunk = Chunk { coords: coords.clone() };
        // Link the components
        entity.link_component::<Chunk>(data.component_manager, chunk).unwrap();
        // Transform
        entity
            .link_component::<components::Transform>(
                data.component_manager,
                components::Transform {
                    position: veclib::Vector3::<f32>::from(coords.position),
                    scale: veclib::Vector3::new((coords.size / octree_size) as f32, (coords.size / octree_size) as f32, (coords.size / octree_size) as f32),
                    ..components::Transform::default()
                },
            )
            .unwrap();
        // Multi Material Renderer
        let mut mm_renderer = MultiMaterialRenderer::default().set_materials(bound_materials.clone());
        // Add the sub models into the Multi Material renderer
        for (material_id, sub_model) in tmodel.shader_model_hashmap {
            mm_renderer = mm_renderer.add_submodel_m(sub_model, material_id as usize);
        }
        for (material_id, skirt_model) in tmodel.skirt_models {
            // Don't forget the skirts
            mm_renderer = mm_renderer.add_submodel_m(skirt_model, material_id as usize);
        }
        // Refresh the data
        mm_renderer.refresh_sub_models();
        let renderer = Renderer::new().set_wireframe(true).set_multimat(mm_renderer);
        entity.link_component::<Renderer>(data.component_manager, renderer).unwrap();
        // Create the AABB
        let aabb = components::AABB::from_components(&entity, data.component_manager);
        entity.link_component::<components::AABB>(data.component_manager, aabb).unwrap();
        let entity_id = data.entity_manager.add_entity_s(entity);
        added_chunk_entities_ids.push((entity_id, coords.clone()));
    }

    // Reassign
    let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    for (entity_id, coords) in added_chunk_entities_ids {
        td.chunk_manager.add_chunk_entity(&coords, entity_id);
    }

    for entity_id in removed_chunks {
        // Removal the entity from the world
        data.entity_manager.remove_entity_s(entity_id).unwrap();
    }

    for node in system.nodes.iter() {
        let debug: debug::DefaultDebugRendererType = debug::DefaultDebugRendererType::CUBE(node.get_center().into(), veclib::Vector3::<f32>::ONE * (node.half_extent as f32) * 2.0);
        if node.children_indices.is_some() {
            //data.debug.renderer.debug_default(debug, veclib::Vector3::ONE, false);
        }
    }

    // Update stats
    system.terrain_stats.best_update_speed = system.terrain_stats.best_update_speed.min(t.elapsed().as_millis());
    system.terrain_stats.worst_update_speed = system.terrain_stats.worst_update_speed.max(t.elapsed().as_millis());
}
fn entity_added(_system_data: &mut SystemData, entity: &Entity, data: &mut WorldData) {
    // Setup the voxel generator for this generator
    let td = entity.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    // Generate the voxel texture
    td.voxel_generator.setup_voxel_generator();
}

// Create the terrain system
pub fn system(data: &mut WorldData) -> System {
    let mut system = System::new();
    // Link the components
    system.link_component::<components::TerrainData>(data.component_manager).unwrap();
    data.component_manager.register_component::<Chunk>();
    // Attach the events
    system.event(SystemEventType::EntityAdded(entity_added));
    system.event(SystemEventType::EntityUpdate(entity_update));
    system.event(SystemEventType::SystemPrefire(system_prefire));
    // Create debug commands
    let command = debug::Command {
        name: "toggle-terrain-gen".to_string(),
        inputs: Vec::new(),
    };
    data.debug.console.register_template_command(command);
    let command = debug::Command {
        name: "stat-terrain".to_string(),
        inputs: Vec::new(),
    };
    data.debug.console.register_template_command(command);
    // Add the custom data
    system.custom_data(CustomData {
        lod_factor: terrain::DEFAULT_LOD_FACTOR,
        nodes: Vec::new(),
        terrain_gen: true,
        terrain_stats: TerrainStats::default(),
    });
    system
}
