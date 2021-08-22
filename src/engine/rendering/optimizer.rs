use crate::engine::core::{defaults::components::{components, transforms}, ecs::{component::FilteredLinkedComponents, entity::Entity, system::{EntityFilter, EntityFilterDataType, EntityFilterWrapper}, system_data::{SystemEventData, SystemEventDataLite}}, world::CustomWorldData};

use super::renderer::Renderer;

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
#[derive(Default)]
pub struct RenderOptimizer {
}

impl EntityFilterWrapper for RenderOptimizer {
    // Create the filter
    fn create_entity_filter(&self) -> EntityFilter {
        return EntityFilter {
            filter_entity_fn: |data | {
                // Returns if an entity should be kept or not
                let overwrite = data.get_bool(0);
                return true;
            },
            get_efdt: |entity, data| {
                // Returns the data types that are going to be used
                let camera_entity = data.entity_manager.id_get_entity(&data.custom_data.main_camera_entity_id).unwrap();
                let camera = camera_entity.get_component::<components::Camera>(data.component_manager).unwrap();
                let aabb_bound = entity.get_component::<components::AABB>(data.component_manager).unwrap();

                vec![
                    EntityFilterDataType::BOOL(entity.entity_id == data.custom_data.sky_entity_id),
                    EntityFilterDataType::AABB(aabb_bound.aabb),
                    EntityFilterDataType::MAT4(camera.frustum_culling_matrix)
                ]
            }   
        }
    }
}

