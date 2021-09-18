use crate::{chunk_data::ChunkCoords, ChunkData, ChunkManager};

use super::voxel::VoxelGenerator;
use debug::DefaultDebugRendererType;
use defaults::components;
use ecs::*;
use input::*;
use math;
use others::CacheManager;
use rendering::*;
use system_event_data::{SystemEventData, SystemEventDataLite};
use systems::*;
use math::octree;
use std::collections::HashMap;

// The actual chunk size number that you change
pub const MAIN_CHUNK_SIZE: usize = 32;
// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = MAIN_CHUNK_SIZE + 2;
// An LOD bias used to change how how high detail chunks spawn
pub const LOD_FACTOR: f32 = 0.5;
// The octree depth
pub const OCTREE_DEPTH: u8 = 8;

// A component that will be added to well... chunks
#[derive(Default)]
pub struct Chunk {
    pub coords: ChunkCoords,
}

// Main traits implemented
ecs::impl_component!(Chunk);

// Hehe terrain generator momenta
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    // Chunk managing
    pub octree: math::octree::Octree,
    pub chunk_manager: ChunkManager,
    // The voxel generator
    pub voxel_generator: VoxelGenerator,
    // Preloaded resources for chunks
    pub shader_name: String,
    pub texture_ids: Vec<u16>,
    parent_child_count: HashMap<veclib::Vector3::<i64>, u8>,
}

impl Terrain {
    // Create a chunk entity
    pub fn add_chunk_entity(&self, texture_cacher: &CacheManager<Texture2D>, component_manager: &mut ComponentManager, coords: &ChunkCoords, model: Model) -> Entity {
        // Create the entity
        let name = format!("Chunk {:?} {:?}", coords.position, coords.size);
        let mut entity = Entity::new(name.as_str());

        // Create the chunk component
        let chunk = Chunk { coords: coords.clone() };
        // Link the components
        entity.link_component::<Chunk>(component_manager, chunk).unwrap();
        entity
            .link_component::<components::Transform>(
                component_manager,
                components::Transform {
                    position: veclib::Vector3::<f32>::from(coords.position),
                    scale: veclib::Vector3::new(
                        (coords.size / self.octree.size) as f32,
                        (coords.size / self.octree.size) as f32,
                        (coords.size / self.octree.size) as f32,
                    ),
                    ..components::Transform::default()
                },
            )
            .unwrap();
        let material = Material::default()
            .set_uniform("uv_scale", ShaderArg::V2F32(veclib::Vector2::<f32>::ONE * 0.05))
            .set_uniform("normals_strength", ShaderArg::F32(1.0))
            .set_shader(self.shader_name.as_str())
            .set_uniform("depth_level", ShaderArg::F32(coords.depth as f32 / OCTREE_DEPTH as f32));
        entity
            .link_component::<Renderer>(component_manager, Renderer::new().set_model(model).set_wireframe(true).set_material(material))
            .unwrap();
        // TODO: Fix this
        entity
            .link_component::<components::AABB>(component_manager, components::AABB::from_components(&entity, component_manager))
            .unwrap();

        return entity;
    }
}

impl System for Terrain {
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
        self.system_data.link_component::<Chunk>(data.component_manager).unwrap();
        self.system_data.link_component::<components::Transform>(data.component_manager).unwrap();
        self.system_data.link_component::<Renderer>(data.component_manager).unwrap();
        self.system_data.link_component::<components::AABB>(data.component_manager).unwrap();

        // Load the shader name
        self.shader_name = Shader::new(
            vec!["defaults\\shaders\\default.vrsh.glsl", "defaults\\shaders\\triplanar.frsh.glsl"],
            data.resource_manager,
            data.shader_cacher,
            None,
        )
        .1;

        // Load the texture ids
        self.texture_ids = vec![
            Texture2D::new()
                .enable_mipmaps()
                .load_texture("user\\textures\\sandstone_cracks_diff_4k.png", data.resource_manager, data.texture_cacher)
                .unwrap()
                .1,
            Texture2D::new()
                .enable_mipmaps()
                .load_texture("user\\textures\\sandstone_cracks_nor_gl_4k.png", data.resource_manager, data.texture_cacher)
                .unwrap()
                .1,
        ];

        // Setup the octree
        self.octree.size = CHUNK_SIZE as u64 - 2;
        self.octree.depth = OCTREE_DEPTH;
        self.octree.lod_factor = LOD_FACTOR;
        // Debug controls
        data.input_manager.bind_key(Keys::Y, "update_terrain", MapType::Button);

        // Load the compute shader for the voxel generator
        self.voxel_generator.compute_shader_name = Shader::new(
            vec!["user\\shaders\\voxel_generator.cmpt.glsl"],
            data.resource_manager,
            data.shader_cacher,
            Some(AdditionalShader::Compute(ComputeShader::default())),
        )
        .1;
        // Generate the voxel texture
        self.voxel_generator.setup_voxel_generator(data);
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        // Get the camera location
        let camera_entity = data
        .entity_manager
        .get_entity(&data.custom_data.main_camera_entity_id)
        .unwrap();
        let camera_location = camera_entity
            .get_component::<components::Transform>(data.component_manager)
            .unwrap()
            .position;
        let camera_forward_vector = camera_entity.get_component::<components::Transform>(data.component_manager).unwrap().rotation.mul_point(veclib::Vector3::Z);

        // Generate the octree each frame and generate / delete the chunks
        if data.input_manager.map_toggled("update_terrain") && self.chunk_manager.octree_update_valid() {
            match self.octree.generate_incremental_octree(camera_location) {
                Some((mut added, removed, added_nodes_hashmap)) => {                    
                    // Calculate the child leaf node count for each node in the post process tree
                    let mut nodes_parents_children_leaf_count: HashMap<veclib::Vector3::<i64>, u8> = HashMap::new();
                    for (node_coords, node) in added_nodes_hashmap.iter() {
                        // Make sure the key exists for our parent
                        nodes_parents_children_leaf_count.entry(node.parent_center).or_insert(0);
                        // Discard child nodes that have children
                        if !node.children {
                            nodes_parents_children_leaf_count.entry(node.parent_center).and_modify(|x| *x += 1);
                        }
                    }
                    self.parent_child_count = nodes_parents_children_leaf_count;
                    // Turn all the newly added nodes into chunks and instantiate them into the world
                    for octree_node in added {
                        // Only add the octree nodes that have no children
                        if !octree_node.children {
                            // Add the chunk in the chunk manager
                            self.chunk_manager.add_chunk(ChunkCoords::new(&octree_node));
                        }
                    }
                    // Delete all the removed octree nodes from the world
                    for octree_node in removed {
                        let chunk_coords = ChunkCoords::new(&octree_node);
                        // Remove the chunk from the chunk manager
                        match self.chunk_manager.remove_chunk(&chunk_coords) {
                            Some(_) => {
                                // Get the entity id
                                self.chunk_manager.remove_chunk_entity(&chunk_coords);
                            }
                            None => {}
                        }
                    }
                }
                None => { /* Nothing happened */ }
            }
            let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
            self.chunk_manager.update_camera_view(&camera_entity, data.component_manager);
        }

        // Update the chunk manager
        //println!("{:?}", self.parent_child_count);        
        let (added_chunks, removed_chunks) = self.chunk_manager.update(&self.voxel_generator, data, self.parent_child_count.clone());
        for (coords, model) in added_chunks {
            // Add the entity
            let entity = self.add_chunk_entity(&data.texture_cacher, &mut data.component_manager, &coords, model);
            let entity_id = data.entity_manager.add_entity_s(entity);
            self.chunk_manager.add_chunk_entity(&coords, entity_id);
        }
        for entity_id in removed_chunks {
            // Removal the entity from the world
            data.entity_manager.remove_entity_s(&entity_id).unwrap();
        }        
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {}

    // When a chunk gets added to the world
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {}

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
