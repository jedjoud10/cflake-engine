use super::voxel::VoxelGenerator;
use hypo_defaults::components;
use hypo_ecs::*;
use hypo_input::*;
use hypo_math as math;
use hypo_others::CacheManager;
use hypo_rendering::*;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::*;
use std::collections::HashMap;

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 18;
// An LOD bias used to change how how high detail chunks spawn
pub const LOD_FACTOR: f32 = 3.0;
// The octree depth
pub const OCTREE_DEPTH: u8 = 8;

// A component that will be added to well... chunks
#[derive(Default)]
pub struct Chunk {
    pub chunk_data: super::ChunkData,
}

// Main traits implemented
hypo_ecs::impl_component!(Chunk);

// This is a procedural model generator
impl ProceduralModelGenerator for Chunk {
    // Generate a procedural marching cube model
    fn generate_model(&self) -> Model {
        super::mesher::generate_model(&self.chunk_data.data, true)
    }
}


// Hehe terrain generator momenta
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    // Terrain generation
    pub voxel_generator: VoxelGenerator,

    // Chunk managing
    pub octree: math::octree::Octree,
    pub chunks: HashMap<veclib::Vector3<i64>, u16>,

    // Preloaded resources for chunks
    pub shader_name: String,
    pub texture_ids: Vec<u16>,
}

impl Terrain {
    // Create a chunk entity
    pub fn add_chunk_entity(&self, texture_cacher: &CacheManager<Texture>, component_manager: &mut ComponentManager, position: veclib::Vector3<i64>, size: u64) -> Option<Entity> {
        // Create the entity
        let name = format!("Chunk {:?} {:?}", position, size);
        let mut entity = Entity::new(name.as_str());

        // Create the chunk component
        let mut chunk = Chunk::default();
        chunk.chunk_data.position = position;
        chunk.chunk_data.size = size;
        let min_max = chunk.chunk_data.generate_data(&self.voxel_generator);
        // Check if we should even generate the model
        if min_max.0.signum() == min_max.1.signum() {
            // No intersection
            return None;
        }

        let model = chunk.generate_model();

        // Link the components
        entity.link_component::<Chunk>(component_manager, chunk).unwrap();
        entity
            .link_component::<components::Transform>(
                component_manager,
                components::Transform {
                    position: veclib::Vector3::<f32>::from(position),
                    scale: veclib::Vector3::new((size / self.octree.size) as f32, (size / self.octree.size) as f32, (size / self.octree.size) as f32),
                    ..components::Transform::default()
                },
            )
            .unwrap();
        entity
            .link_component::<Renderer>(
                component_manager,
                Renderer::new()
                    .load_textures(self.texture_ids.clone(), texture_cacher)
                    .set_model(model)
                    .set_uv_scale(veclib::Vector2::<f32>::default_one() * 0.05)
                    .set_wireframe(true)
                    .set_shader(self.shader_name.as_str()),
            )
            .unwrap();
        // TODO: Fix this
        entity
            .link_component::<components::AABB>(component_manager, components::AABB::from_components(&entity, component_manager))
            .unwrap();

        return Some(entity);
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
        )
        .1;

        // Load the texture ids
        self.texture_ids = vec![
            Texture::new()
                .enable_mipmaps()
                .load_texture("user\\textures\\rock\\Rock033_1K_Color.png", data.resource_manager, data.texture_cacher)
                .unwrap()
                .1,
            Texture::new()
                .enable_mipmaps()
                .load_texture("user\\textures\\rock\\Rock033_1K_Normal.png", data.resource_manager, data.texture_cacher)
                .unwrap()
                .1,
        ];

        // Setup the octree
        self.octree.size = CHUNK_SIZE as u64 - 2;
        self.octree.depth = OCTREE_DEPTH;
        self.octree.lod_factor = LOD_FACTOR;
        let nodes = self.octree.generate_base_octree();
        /*
        if let Option::Some(chunk_entity) = self.add_chunk_entity(data.texture_cacher, data.component_manager, veclib::Vector3::default_zero(), 16) {
            let entity_id = data.entity_manager.add_entity_s(chunk_entity);
        }
        */
        for (_, octree_node) in nodes {
            // Only add the octree nodes that have no children
            if !octree_node.children {                
                let chunk_entity = self.add_chunk_entity(data.texture_cacher, data.component_manager, octree_node.position, octree_node.half_extent * 2);
                if let Option::Some(chunk_entity) = chunk_entity {
                    let entity_id = data.entity_manager.add_entity_s(chunk_entity);
                    self.chunks.insert(octree_node.get_center(), entity_id);
                }                
            }
        }
        // Debug controls
        data.input_manager.bind_key(Keys::Y, "update_terrain", MapType::Button);
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        // Get the camera location
        let camera_location = data
            .entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<components::Transform>(data.component_manager)
            .unwrap()
            .position;

        // Generate the octree each frame and generate / delete the chunks
        if data.input_manager.map_toggled("update_terrain") {
            self.octree.generate_incremental_octree(math::octree::OctreeInput { target: camera_location });

            // Turn all the newly added nodes into chunks and instantiate them into the world
            for octree_node in &self.octree.added_nodes {
                // Only add the octree nodes that have no children
                if !octree_node.children {
                    if !self.chunks.contains_key(&octree_node.get_center()) {
                        let chunk_entity = self.add_chunk_entity(data.texture_cacher, data.component_manager, octree_node.position, octree_node.half_extent * 2);
                        match chunk_entity {
                            Some(chunk_entity) => {
                                let entity_id = data.entity_manager.add_entity_s(chunk_entity);
                                self.chunks.insert(octree_node.get_center(), entity_id);
                            }
                            _ => {}
                        }
                    }
                }
            }
            // Delete all the removed octree nodes from the world
            for octree_node in &self.octree.removed_nodes {
                if self.chunks.contains_key(&octree_node.get_center()) {
                    // Remove the chunk from our chunks and from the world
                    let entity_id = self.chunks.remove(&octree_node.get_center()).unwrap();
                    data.entity_manager.remove_entity_s(&entity_id).unwrap();
                }
            }
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
