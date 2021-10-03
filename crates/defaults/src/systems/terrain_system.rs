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
    // Debug elements ID
    element_id: u16,
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
        let mut root = ui::Root::new();
        let root_elem = ui::Element::default();
        // Add the element to the root node
        root.add_element(root_elem);

        // Text for chunk debug data
        let elem = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::Y * 60.0 * 3.0)
            .set_text("chunk_data_here", 60.0);
        self.element_id = root.add_element(elem);
        data.ui_manager.add_root("terrain_debug", root);
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
            match td.octree.generate_incremental_octree(camera_location) {
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
            let material = clone_material.clone()
                .set_uniform("uv_scale", ShaderArg::V2F32(veclib::Vector2::<f32>::ONE * 0.02))
                .set_uniform("normals_strength", ShaderArg::F32(1.0));
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

        for entity_id in removed_chunks {
            // Removal the entity from the world
            data.entity_manager.remove_entity_s(&entity_id).unwrap();
        }

        // Update the UI debug chunk data        
        let root = data.ui_manager.get_root_mut("terrain_debug");
        let text = &format!("Chunks to generate: {}", td.chunk_manager.chunks_to_generate.len());
        root.get_element_mut(self.element_id).update_text(text, 60.0);        
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
