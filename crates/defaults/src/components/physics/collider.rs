use world::{ecs::component::Component, math::shapes::ShapeType, rendering::basics::mesh::Mesh};

// Collider components
#[derive(Component)]
pub struct Collider {
    // Parameters
    pub restitution: f32,
    pub friction: f32,

    // Collider type
    pub _type: ColliderType,
}

// Collider type
pub enum ColliderType {
    Shape(ShapeType),
    Mesh(Mesh),
}
