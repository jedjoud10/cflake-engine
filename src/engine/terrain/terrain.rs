use crate::engine::{core::{defaults::components, ecs::{component::{ComponentManager, FilteredLinkedComponents}, entity::Entity, system::System, system_data::{SystemData, SystemEventData, SystemEventDataLite}}}, debug, math::{self, octree::OctreeInput}, rendering::{model::ProceduralModelGenerator, renderer::Renderer, shader::Shader}, terrain::chunk::Chunk};

use super::voxel::VoxelGenerator;

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 32;

// Hehe terrain generator moment
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    pub isoline: f32,
    pub octree: math::octree::Octree,
    pub voxel_generator: VoxelGenerator,
}

impl Terrain {
    // Create a chunk entity
    pub fn add_chunk_entity(&self, component_manager: &mut ComponentManager, position: glam::IVec3, size: u16) -> Entity {
        // Create the entity
        let mut chunk = Entity::new(format!("Chunk {:?} {:?}", position, size).as_str());

        // Create the chunk component
        let mut chunk_cmp = Chunk::default();
        chunk_cmp.position = position;
        chunk_cmp.size = size;
        chunk_cmp.generate_data(&self.voxel_generator);
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

        self.octree.size = 32;   
        self.octree.depth = 4;
        self.octree.generate_octree(OctreeInput { camera: math::shapes::Sphere {
            center: glam::Vec3::ONE,
            radius: 1.0,
        }});
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        // Get the camera location
        let camera_location = data.entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id).unwrap()
            .get_component::<components::Transform>(data.component_manager).unwrap();
        // Generate the octree each frame and generate / delete the chunks     
        
        
        
        //data.debug.debug_default(debug::DefaultDebugRendererType::AABB(octree_node.get_aabb()));

        // Add all the new nodes as new chunks
        for octree_node in &self.octree.added_nodes {
            let chunk_entity = self.add_chunk_entity(data.component_manager, octree_node.position, octree_node.extent);
            data.entity_manager.add_entity_s(chunk_entity);
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
