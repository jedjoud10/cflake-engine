use crate::engine::core::{ecs::{component::FilteredLinkedComponents, entity::Entity, system::EntityPrePassFilter}, world::CustomWorldData};

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
#[derive(Default)]
pub struct RenderOptimizer {}

impl EntityPrePassFilter for RenderOptimizer {
    // Filter the entity based on it's visibilit
    fn filter_entity(&self, entity: &Entity, flc: &FilteredLinkedComponents, custom_data: &CustomWorldData) -> bool {
        entity.entity_id == custom_data.sky_entity_id
    }
}
