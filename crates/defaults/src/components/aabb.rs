use main::{ecs::component::Component, math};
// An AABB component
#[derive(Default, Component)]
pub struct AABB {
    pub aabb: math::bounds::AABB,
}

// AABB component functions
impl AABB {
    // Offset a pre-existing AABB with a transform
    pub fn offset(mut aabb: math::bounds::AABB, transform: &super::Transform) -> math::bounds::AABB {
        // Offset the shit
        aabb.min += transform.position;
        aabb.max += transform.position;
        // Scale it, the position is the center
        aabb.scale(transform.position, transform.scale);
        // Recalculate the center
        aabb.center = (aabb.min + aabb.max) / 2.0;
        aabb
    }
}
