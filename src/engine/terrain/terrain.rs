use crate::engine::{core::{defaults::components::{components, transforms}, ecs::{
            component::{Component, ComponentID, FilteredLinkedComponents},
            entity::Entity,
            system::System,
            system_data::{SystemData, SystemEventData, SystemEventDataLite},
        }}, math::bounds, rendering::{model::ProceduralModelGenerator, renderer::Renderer, shader::Shader}, terrain::chunk::Chunk};
use std::collections::HashMap;

// How many voxels in one axis in each chunk?
pub const CHUNK_SIZE: usize = 32;

// Hehe terrain generator moment
pub struct Terrain {
    pub system_data: SystemData,
    pub chunks: Vec<glam::IVec3>,
    pub isoline: f32,
    pub camera_chunk_position: glam::IVec3,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            system_data: SystemData::default(),
            chunks: Vec::new(),
            isoline: 0.0,
            camera_chunk_position: glam::IVec3::ZERO,
        }
    }
}

// All the terrain generator code
impl Terrain {
    // Density functions
    fn density(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut density: f32 = glam::vec3(x, y, z).length() - 10.0;
        density = density.min(y + (x * 0.2).sin() + (z * 0.2).sin()) - 10.0;
        density
    }
    // Creates a single chunk entity
    fn create_single_chunk(&mut self, position: glam::Vec3, data: &mut SystemEventData) -> u16 {
        let now = std::time::Instant::now();
        // Generate the component
        let mut chunk = Chunk::default();
        chunk.position = position;
        chunk.generate_data(self);

        // Generate the model
        let model = chunk.generate_model();

        // Create the entity
        let mut chunk_entity = Entity::new("Chunk");

        // Load the renderer
        let mut rc = Renderer::default();
        rc.shader_name = Shader::new(
            vec!["shaders\\default.vrsh.glsl", "shaders\\triplanar.frsh.glsl"],
            &mut data.resource_manager,
            &mut data.shader_cacher,
        )
        .1;
        rc.model = model;
        // Load the terrain textures
        rc.resource_load_textures(
            vec!["textures\\rock\\Rock033_1K_Color.png", "textures\\rock\\Rock033_1K_Normal.png"],
            &mut data.texture_cacher,
            &mut data.resource_manager,
        );
        rc.uv_scale = glam::vec2(0.2, 0.2);

        // Link the required components to the entity
        chunk_entity.link_component::<components::AABB>(data.component_manager, components::AABB::from_model(&rc.model, position)).unwrap();
        chunk_entity.link_component::<Renderer>(data.component_manager, rc).unwrap();
        chunk_entity
            .link_component::<transforms::Position>(data.component_manager, transforms::Position { position })
            .unwrap();
        chunk_entity.link_default_component::<transforms::Rotation>(data.component_manager).unwrap();
        chunk_entity.link_default_component::<transforms::Scale>(data.component_manager).unwrap();

        // This is in global coordinates btw (-30, 0, 30, 60)
        self.chunks.push(position.as_i32());
        // Add the entity to the world
        println!("{} ms to generate chunk entity", now.elapsed().as_millis());
        data.entity_manager.add_entity_s(chunk_entity)
    }
    // 1. Create the chunks, and generate their data
    // 2. Create the actual chunk entities and create the models
    pub fn generate_terrain(&mut self, data: &mut SystemEventData) {
        self.isoline = 0.0;
        // Create the entity
        for x in -2..2 {
            for y in 0..2 {
                for z in -2..2 {
                    let position = glam::vec3(
                        ((CHUNK_SIZE as f32) - 2.0) * x as f32,
                        ((CHUNK_SIZE as f32) - 2.0) * y as f32,
                        ((CHUNK_SIZE as f32) - 2.0) * z as f32,
                    );
                    self.create_single_chunk(position, data);
                }
            }
        }
    }
    // When we want to update the terrain
    pub fn update_terrain(&mut self, position: glam::Vec3, _data: &mut SystemEventData) {
        let new_camera_chunk_position = ((position - 2.0) / CHUNK_SIZE as f32).floor().as_i32();
        if new_camera_chunk_position != self.camera_chunk_position {
            // The camera moved from one chunk to another
        }
        self.camera_chunk_position = new_camera_chunk_position;
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
        // This system will loop over all the chunks and generate new ones if needed
        self.system_data.link_component::<Chunk>(data.component_manager).unwrap();
        self.system_data.link_component::<Renderer>(data.component_manager).unwrap();
        self.system_data.link_component::<transforms::Position>(data.component_manager).unwrap();
        self.generate_terrain(data);
    }

    // Update the camera position inside the terrain generator
    fn pre_fire(&mut self, data: &mut SystemEventData) {
        let camera_position = data
            .entity_manager
            .get_entity(&data.custom_data.main_camera_entity_id)
            .unwrap()
            .get_component::<transforms::Position>(data.component_manager)
            .unwrap()
            .position;
        self.update_terrain(camera_position, data);
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {}

    // When a chunk gets added to the world
    fn entity_added(&mut self, entity: &Entity, data: &mut SystemEventDataLite) {
        // Generate the data for this chunk and then create the model
        let chunk = entity.get_component_mut::<Chunk>(data.component_manager).unwrap();
        chunk.generate_data(self);
        let model = chunk.generate_model();
        let rc = entity.get_component_mut::<Renderer>(data.component_manager).unwrap();
        rc.model = model;
        rc.refresh_model();
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
