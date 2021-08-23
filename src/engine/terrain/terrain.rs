use crate::engine::{core::{
        defaults::components,
        ecs::{
            component::FilteredLinkedComponents,
            entity::Entity,
            system::System,
            system_data::{SystemData, SystemEventData, SystemEventDataLite},
        },
    }, math, rendering::{model::ProceduralModelGenerator, renderer::Renderer, shader::Shader}, terrain::chunk::Chunk};

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 32;

// Hehe terrain generator moment
#[derive(Default)]
pub struct Terrain {
    pub system_data: SystemData,
    pub isoline: f32,
    pub octree: math::octree::Octree
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
        self.octree.generate_octree(math::octree::OctreeInput { camera: math::shapes::Sphere { center: glam::Vec3::ZERO, radius: 5.0 } });
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {}

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
