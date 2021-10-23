use debug::DefaultDebugRendererType;
use terrain::{BoundChecker, ChunkCoords};

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
}
crate::impl_custom_system_data!(CustomData);

// Events
fn system_prefire(system_data: &mut SystemData, data: &mut WorldData) {
    let system = system_data.cast_mut::<CustomData>().unwrap();
    // Update the LOD factor using the commands
    match data.debug.console.listen_command("terrain-set-lod-factor") {
        Some(x) => match x.get_input("-v") {
            Some(x) => match x {
                debug::CommandInputEnum::F32(x) => system.lod_factor = *x,
                _ => {}
            },
            _ => {}
        },
        None => {}
    }
}
fn entity_update(system_data: &mut SystemData, _entity: &Entity, components: &FilteredLinkedComponents, data: &mut WorldData) {
    // Get the camera location
    let camera_entity = data.entity_manager.get_entity(data.custom_data.main_camera_entity_id).unwrap();
    let camera_transform = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap();
    let system = system_data.cast_mut::<CustomData>().unwrap();
    // Get the camera transform values
    let camera_location = camera_transform.position;
    let camera_forward_vector = camera_transform.get_forward_vector();

    // Get the terrain data
    let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    let octree_size = td.octree.internal_octree.size;
    let clone_material = td.material.clone();

    // Generate the octree each frame and generate / delete the chunks
    if data.debug.console.listen_command("toggle-terrain-gen").is_some() {
        system.terrain_gen = !system.terrain_gen;
    }
    if td.chunk_manager.octree_update_valid() && system.terrain_gen {
        match td.octree.generate_incremental_octree(&camera_location, system.lod_factor) {
            Some((mut added, removed)) => {
                system.nodes.clear();
                // Filter first
                added.retain(|node| BoundChecker::bound_check(&node));
                // Turn all the newly added nodes into chunks and instantiate them into the world
                for octree_node in added {
                    // Only add the octree nodes that have no children
                    if octree_node.children_indices.is_none() {
                        // Add the chunk in the chunk manager
                        td.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                    }
                }
                // Delete all the removed octree nodes from the world
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
    let (added_chunks, removed_chunks) = td
        .chunk_manager
        .update(&td.voxel_generator, &mut data.shader_cacher.1, data.time_manager.frame_count);
    let mut added_chunk_entities_ids: Vec<(usize, ChunkCoords)> = Vec::new();
    let depth = td.octree.internal_octree.depth as f32;

    // Add the entities to the entity manager
    for (coords, model) in added_chunks {
        // Add the entity
        let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
        let mut entity = Entity::new(name.as_str());

        // Create the chunk component
        let chunk = Chunk { coords: coords.clone() };
        // Link the components
        entity.link_component::<Chunk>(data.component_manager, chunk).unwrap();
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

        let material = clone_material.clone();
        entity
            .link_component::<Renderer>(data.component_manager, Renderer::new().set_model(model).set_wireframe(true).set_material(material))
            .unwrap();

        let entity_id = data.entity_manager.add_entity_s(entity);
        added_chunk_entities_ids.push((entity_id, coords.clone()));
    }

    // Reassign
    let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
    for (entity_id, coords) in added_chunk_entities_ids {
        td.chunk_manager.add_chunk_entity(&coords, entity_id);
    }

    for coords in td.chunk_manager.chunks_to_generate.iter() {
        let size = veclib::Vector3::<f32>::from(veclib::Vector3::ONE * coords.size);
        let debug = DefaultDebugRendererType::CUBE(coords.center.into(), size);
        data.debug.renderer.debug_default(debug, veclib::Vector3::ONE, false);
    }

    for entity_id in removed_chunks {
        // Removal the entity from the world
        data.entity_manager.remove_entity_s(entity_id).unwrap();
    }

    for node in system.nodes.iter() {
        let debug: debug::DefaultDebugRendererType = debug::DefaultDebugRendererType::CUBE(node.get_center().into(), veclib::Vector3::<f32>::ONE * (node.half_extent as f32) * 2.0);
        if node.children_indices.is_some() {
            data.debug.renderer.debug_default(debug, veclib::Vector3::ONE, false);
        } else {
        }
    }
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
        name: "terrain-set-lod-factor".to_string(),
        inputs: vec![debug::CommandInput::new::<f32>("-v")],
    };
    data.debug.console.register_template_command(command);
    let command = debug::Command {
        name: "toggle-terrain-gen".to_string(),
        inputs: Vec::new(),
    };
    data.debug.console.register_template_command(command);
    // Add the custom data
    system.custom_data(CustomData {
        lod_factor: terrain::DEFAULT_LOD_FACTOR,
        nodes: Vec::new(),
        terrain_gen: true,
    });
    system
}
