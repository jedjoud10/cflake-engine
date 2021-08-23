use crate::engine::{core::{
    defaults::components,
    ecs::{component::FilteredLinkedComponents, entity::Entity, system::EntityFilter, system_data::SystemEventData},
}, math::Intersection};

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
#[derive(Default)]
pub struct RenderOptimizer {}

impl EntityFilter for RenderOptimizer {
    // Filter the entity based on it's visibilit
    fn filter_entity(&self, entity: &Entity, components: &FilteredLinkedComponents, data: &SystemEventData) -> bool {
        let mut render_entity: bool;
        // Just to help make the code a bit cleaner
        let camera_entity = data.entity_manager.get_entity(&data.custom_data.main_camera_entity_id).unwrap();
        let camera = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
        let aabb = components.get_component::<components::AABB>(data.component_manager).unwrap().aabb;

        // Always render the sky
        if entity.entity_id == data.custom_data.sky_entity_id {
            return true;
        }
        // Don't render the entity if the camera cannot see it
        let intersection = Intersection::frustum_aabb(&camera.frustum, &aabb);
        render_entity = intersection;
        //render_entity
        return true;
    }
}
