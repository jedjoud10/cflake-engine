use std::collections::HashMap;
use crate::engine::{self, core::{cacher::CacheManager, defaults::components, ecs::{component::{ComponentManager, FilteredLinkedComponents}, entity::Entity, system::System, system_data::{SystemData, SystemEventData, SystemEventDataLite}}}, debug, input, math::{self, octree::OctreeInput}, rendering::{model::{Model, ProceduralModelGenerator}, renderer::Renderer, shader::Shader, texture::Texture}, terrain::chunk::Chunk};
use super::voxel::VoxelGenerator;

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 18;
// An LOD bias used to change how how high detail chunks spawn
pub const LOD_THRESHOLD: f32 = 1.2;
// The octree depth
pub const OCTREE_DEPTH: u8 = 4;

// Hehe terrain generator moment
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    // Terrain generation
    pub voxel_generator: VoxelGenerator,
    
    // Chunk managing
    pub octree: math::octree::Octree,
    pub chunks: HashMap<glam::IVec3, u16>,

    // Preloaded resources for chunks
    pub shader_name: String,
    pub texture_ids: Vec<u16>,
}

impl Terrain {
    // Create a chunk entity
    pub fn add_chunk_entity(&self, texture_cacher: &CacheManager<Texture>, component_manager: &mut ComponentManager, position: glam::IVec3, size: u16) -> Option<Entity> {
        // Create the entity
        let mut chunk = Entity::new(format!("Chunk {:?} {:?}", position, size).as_str());

        // Create the chunk component
        let mut chunk_cmp = Chunk::default();
        chunk_cmp.position = position;
        chunk_cmp.size = size;
        let min_max = chunk_cmp.generate_data(&self.voxel_generator);
        // Check if we should even generate the model
        if min_max.0.signum() == min_max.1.signum() {
            // No intersection
            return None;
        }
        let model = chunk_cmp.generate_model();

        // Link the components
        chunk.link_component::<Chunk>(component_manager, chunk_cmp).unwrap();
        chunk.link_component::<components::Transform>(component_manager, components::Transform {
            position: position.as_f32(),
            scale: glam::vec3((size / self.octree.size as u16) as f32, (size / self.octree.size as u16) as f32, (size / self.octree.size as u16) as f32),
            ..components::Transform::default()
        }).unwrap();
        chunk.link_component::<Renderer>(component_manager, Renderer::new()
            .load_textures(self.texture_ids.clone(), texture_cacher)
            .set_model(model)
            .set_shader(self.shader_name.as_str())).unwrap();
        chunk.link_component::<components::AABB>(component_manager, components::AABB::from_components(&chunk, component_manager)).unwrap();
        
        return Some(chunk);
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
        self.shader_name = Shader::new(vec!["shaders\\default.vrsh.glsl", "shaders\\triplanar.frsh.glsl"], data.resource_manager, data.shader_cacher).1;

        // Load the texture ids
        self.texture_ids = vec![
            Texture::new().enable_mipmaps().load_texture("textures\\rock\\Rock033_1K_Color.png", data.resource_manager, data.texture_cacher).unwrap().1,
            Texture::new().enable_mipmaps().load_texture("textures\\rock\\Rock033_1K_Normal.png", data.resource_manager, data.texture_cacher).unwrap().1,
        ];

        // Setup the octree
        self.octree.size = CHUNK_SIZE as u8 - 2;   
        self.octree.depth = OCTREE_DEPTH;   
        self.octree.threshold = LOD_THRESHOLD;
        
        // Debug controls
        data.input_manager.bind_key(input::Keys::Y, "update_terrain", input::MapType::Toggle);
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        // Get the camera location
        let camera_location = data.entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id).unwrap()
            .get_component::<components::Transform>(data.component_manager).unwrap().position;
        // Generate the octree each frame and generate / delete the chunks     
        
        for (_, octree_node) in &self.octree.nodes {
            // Only add the octree nodes that have no children
            if !octree_node.children {
                data.debug.debug_default(debug::DefaultDebugRendererType::AABB(octree_node.get_aabb()));
            }
        }       
        
        if data.input_manager.map_toggled("update_terrain") {   
            // Update the terrain
            self.octree.generate_octree(OctreeInput { target: camera_location });
            // Only do one thing, either add the nodes, or remove them
            if self.octree.added_nodes.len() > 0 {
                // Turn all the newly added nodes into chunks and instantiate them into the world
                for octree_node in &self.octree.added_nodes {
                    // Only add the octree nodes that have no children
                    if !octree_node.children {
                        let chunk_entity = self.add_chunk_entity(data.texture_cacher, data.component_manager, octree_node.position, octree_node.half_extent * 2);
                        if let Option::Some(chunk_entity) = chunk_entity {
                            let entity_id = data.entity_manager.add_entity_s(chunk_entity);
                            self.chunks.insert(octree_node.get_center(), entity_id);
                        }                        
                    }
                }
                self.octree.added_nodes.clear();
            } else if self.octree.removed_nodes.len() > 0 {
                // Delete all the removed octree nodes from the world 
                for octree_node in &self.octree.removed_nodes {
                    if self.chunks.contains_key(&octree_node.get_center()) {
                        // Remove the chunk from our chunks and from the world
                        let entity_id = self.chunks.remove(&octree_node.get_center()).unwrap();
                        data.entity_manager.remove_entity_s(&entity_id).unwrap();
                    }
                }    
                self.octree.removed_nodes.clear();
            }            
        }
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {
    }

    // When a chunk gets added to the world
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
