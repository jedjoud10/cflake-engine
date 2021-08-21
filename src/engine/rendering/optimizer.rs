use crate::engine::core::{defaults::components::{components, transforms}, ecs::{component::FilteredLinkedComponents, entity::Entity, system_data::{SystemEventData, SystemEventDataLite}}, world::CustomWorldData};

use super::renderer::Renderer;
/*
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
        let camera = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
        let aabb_bound = components.get_component::<components::AABB>(data.component_manager).unwrap();
        // Don't render the entity if the camera cannot see it
        render_entity = aabb_bound.aabb.intersect_camera_view_frustum(camera);
        render_entity |= entity.entity_id == data.custom_data.sky_entity_id;
                
        return render_entity;
    }
}

*/