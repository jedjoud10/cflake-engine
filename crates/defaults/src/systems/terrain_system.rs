use debug::DefaultDebugRendererType;
use terrain::{ChunkCoords, BoundChecker, ChunkData, ChunkManager};

use terrain::VoxelGenerator;
use ecs::*;
use input::*;
use math::octree::{self, Octree};
use math::{self, octree::OctreeNode};
use others::CacheManager;
use rendering::*;
use std::collections::{HashMap, HashSet};
use system_event_data::{SystemEventData, SystemEventDataLite};
use systems::*;
use components::Chunk;

use crate::components;

// TODO:
// Gotta make this way, way faster
// Hehe terrain generator momenta
#[derive(Default)]
pub struct TerrainSystem {
    pub system_data: SystemData,
    pub lod_factor: f32,
}

impl System for TerrainSystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        // Link the components
        let system_data = self.get_system_data_mut();
        data.component_manager.register_component::<Chunk>();
        system_data.link_component::<components::TerrainData>(data.component_manager).unwrap();        

        // Create a debug UI for this terrain
        let mut root = ui::Root::new(2); 

        // Create debug commands
        let command = debug::Command {
            name: "terrain-set-lod-factor".to_string(),
            inputs: vec![debug::CommandInput::new::<f32>("-v")],
        };
        data.debug.console.register_template_command(command);

        // Default LOD factor
        self.lod_factor = terrain::DEFAULT_LOD_FACTOR;
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {
        // Get the camera location
        let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
        let camera_transform = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap();
        // Get the camera transform values
        let camera_location = camera_transform.position;
        let camera_forward_vector = camera_transform.get_forward_vector();
        
        // Get the terrain data
        let td = components.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();    
        let octree_size = td.octree.size;    
        let clone_material = td.material.clone();

        // Generate the octree each frame and generate / delete the chunks
        if td.chunk_manager.octree_update_valid() {
            match td.octree.generate_incremental_octree(camera_location, self.lod_factor) {
                Some((mut added, removed, total_nodes)) => {
                    // Filter first
                    added.retain(|_, node| BoundChecker::bound_check(&node));
                    // Turn all the newly added nodes into chunks and instantiate them into the world
                    for (_, octree_node) in added {
                        // Only add the octree nodes that have no children
                        if !octree_node.children {
                            // Add the chunk in the chunk manager
                            td.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                        }
                    }
                    // Delete all the removed octree nodes from the world
                    for (_, octree_node) in removed {
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
        //println!("{:?}", self.parent_child_count);
        // Get the compute shader and frame count
        let compute_shader = data.shader_cacher.1.get_object_mut(&td.voxel_generator.compute_shader_name).unwrap();
        let (added_chunks, removed_chunks) = td.chunk_manager.update(&td.voxel_generator, compute_shader, data.time_manager.frame_count);
        let mut added_chunk_entities_ids: Vec<(u16, ChunkCoords)> = Vec::new();
        let depth = td.octree.depth as f32;
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
                    scale: veclib::Vector3::new(
                        (coords.size / octree_size) as f32,
                        (coords.size / octree_size) as f32,
                        (coords.size / octree_size) as f32,
                    ),
                    ..components::Transform::default()
                },
            )
            .unwrap();
            // TODO: Make a custom material instance system
            let material = clone_material.clone().set_uniform("depth", ShaderArg::F32(coords.depth as f32 / depth));
            entity
                .link_component::<Renderer>(data.component_manager, Renderer::new().set_model(model).set_wireframe(true).set_material(material))
                .unwrap();            
            // TODO: Fix this            
            entity
                .link_component::<components::AABB>(data.component_manager, components::AABB::from_components(&entity, data.component_manager))
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
            data.entity_manager.remove_entity_s(&entity_id).unwrap();
        } 

        // Update the LOD factor using the commands
        match data.debug.console.listen_command("terrain-set-lod-factor") {
            Some(x) => {
                match x.get_input("-v") {
                    Some(x) => match x {
                        debug::CommandInputEnum::F32(x) => self.lod_factor = *x,
                        _ => {}
                    },
                    _ => {}
                }
            },
            None => {},
        }
    }

    // When a terrain generator gets added to the world
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // Setup the voxel generator for this generator
        let td = entity.get_component_mut::<components::TerrainData>(data.component_manager).unwrap();
        // Generate the voxel texture
        td.voxel_generator.setup_voxel_generator();        
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
