use crate::engine::core::ecs::{component::FilteredLinkedComponents, entity::Entity, system::EntityPrePassFilter};

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
pub struct RenderOptimizer {}

impl EntityPrePassFilter for RenderOptimizer {
    // Filter the entity based on it's visibilit
    fn filter_entity(&self, entity: &Entity, flc: &FilteredLinkedComponents) -> bool {
        false
    }
}
