use crate::engine::core::ecs::system::EntityPrePassFilter;

// Optimizes the rendering of objects using multiple techniques like frustum culling and such
pub struct RenderOptimizer {
}

impl EntityPrePassFilter for RenderOptimizer {
    // Filter the entity based on it's visibility
    fn filter_entity(&self, entity: &crate::engine::core::ecs::entity::Entity) -> bool {
        true
    }
}