use ecs::*;
use math;

// An AABB components
#[derive(Default)]
pub struct AABB {
    pub aabb: math::bounds::AABB,
    pub generation_type: AABBGenerationType,
}

// How we are going to generate the AABB
pub enum AABBGenerationType {
    RenderEntity,
    Manual,
}

// Automatically try to load the AABB from the components of a render entity (Position, Scale, Render)
impl Default for AABBGenerationType {
    fn default() -> Self {
        Self::RenderEntity
    }
}

// AABB component functions
impl AABB {
    // Generate the AABB from a renderer entity
    pub fn from_components(entity: &Entity, component_manager: &ComponentManager) -> Self {
        let model_ref = &entity.get_component::<rendering::Renderer>(component_manager).unwrap().model;
        let transform = entity.get_component::<super::Transform>(component_manager).unwrap();
        let mut aabb = math::bounds::AABB::from_model(model_ref.vertices.clone());
        aabb.transform(&transform.get_matrix());
        Self { aabb, ..Self::default() }
    }
}

// Main traits implemented
ecs::impl_component!(AABB);
