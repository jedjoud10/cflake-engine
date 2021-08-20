use crate::engine::core::{defaults::components::transforms, ecs::{component::FilteredLinkedComponents, entity::Entity, system::EntityPrePassFilter, system_data::{SystemEventData, SystemEventDataLite}}, world::CustomWorldData};

use super::renderer::Renderer;

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
#[derive(Default)]
pub struct RenderOptimizer {}

impl EntityPrePassFilter for RenderOptimizer {
    // Filter the entity based on it's visibilit
    fn filter_entity(&self, entity: &Entity, components: &FilteredLinkedComponents, data: &SystemEventData) -> bool {
        // Always render the sky
        let mut render_entity: bool = true;
        // Just to help make the code a bit cleaner
        let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
        let entity_position = components.get_component::<transforms::Position>(data.component_manager).unwrap().position;
        let camera_position = camera_entity.get_component::<transforms::Position>(data.component_manager).unwrap().position;
        let camera_forward = camera_entity.get_component::<transforms::Rotation>(data.component_manager)
            .unwrap()
            .rotation
            .mul_vec3(glam::vec3(0.0, 0.0, 1.0));        
                
        // Don't render the entity if the camera cannot see it
        if ((entity_position - camera_position).normalize()).dot(camera_forward) > -0.8 {
            render_entity = false;
        }
        render_entity |= entity.entity_id == data.custom_data.sky_entity_id;
                
        return render_entity;
    }
}
