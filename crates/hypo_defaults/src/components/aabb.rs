use hypo_math as math;
use hypo_ecs::*;

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
    /*
    // Generate the AABB from a renderer entity
    pub fn from_components(entity: &Entity, component_manager: &ComponentManager) -> Self {
        let model_ref = &entity.get_component::<Renderer>(component_manager).unwrap().model;
        let transform = entity.get_component::<transforms::Transform>(component_manager).unwrap();
        let mut aabb = bounds::AABB::from_model(model_ref);
        aabb.transform(transform);
        Self { aabb, ..Self::default() }
    }
    */
}

// Main traits implemented
impl ComponentInternal for AABB {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for AABB {
    fn get_component_name() -> String {
        String::from("AABB")
    }
}
impl Component for AABB {}
