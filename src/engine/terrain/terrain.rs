use crate::engine::{core::{defaults::components, ecs::{component::{ComponentManager, FilteredLinkedComponents}, entity::Entity, system::System, system_data::{SystemData, SystemEventData, SystemEventDataLite}}}, debug, math::{self, octree::OctreeInput}, rendering::{model::ProceduralModelGenerator, renderer::Renderer, shader::Shader}, terrain::chunk::Chunk};

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 32;

// Hehe terrain generator moment
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    pub isoline: f32,
    pub octree: math::octree::Octree
}

impl Terrain {
    // Create a chunk entity
    pub fn add_chunk_entity(&mut self, component_manager: &mut ComponentManager, position: glam::IVec3, size: u16) -> Entity {
        // Create the entity
        let mut chunk = Entity::new(format!("Chunk {:?} {:?}", position, size).as_str());

        // Create the chunk component
        let mut chunk_cmp = Chunk::default();
        chunk_cmp.position = position;
        chunk_cmp.size = size;
        let model = chunk_cmp.generate_model();

        // Link the components
        chunk.link_component::<Chunk>(component_manager, chunk_cmp).unwrap();
        chunk.link_component::<components::Transform>(component_manager, components::Transform {
            position: position.as_f32(),
            scale: glam::vec3(size as f32, size as f32, size as f32),
            ..components::Transform::default()
        }).unwrap();
        chunk.link_component::<Renderer>(component_manager, Renderer::new().set_model(model)).unwrap();
        chunk.link_component::<components::AABB>(component_manager, components::AABB::from_components(&chunk, component_manager)).unwrap();
        return chunk;
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
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        // Get the camera location
        let camera_location = data.entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id).unwrap()
            .get_component::<components::Transform>(data.component_manager).unwrap();
        let test_location = glam::vec3(data.time_manager.seconds_since_game_start.sin() as f32, data.time_manager.seconds_since_game_start.cos() as f32, data.time_manager.seconds_since_game_start.cos() as f32) * 8.0;
        // Generate the octree each frame and generate / delete the chunks
        self.octree.generate_octree(OctreeInput { camera: math::shapes::Sphere {
            center: test_location,
            radius: 1.0,
        }});
        for (octree_node) in &self.octree.removed_nodes {
            data.debug.debug_default(debug::DefaultDebugRendererType::AABB(octree_node.get_aabb()));
        }
        data.debug.debug_default(debug::DefaultDebugRendererType::CUBE(test_location, glam::Vec3::ONE));
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
