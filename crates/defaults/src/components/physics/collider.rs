use world::{
    ecs::component::Component,
    math::shapes::ShapeType,
    physics::rapier3d::prelude::{ColliderBuilder, ColliderHandle},
    rendering::{basics::mesh::Mesh, object::ObjectID},
};

// Collider components
#[derive(Component)]
pub struct Collider {
    // Handle
    pub(crate) handle: ColliderHandle,

    // Parameters
    pub restitution: f32,
    pub friction: f32,

    // Collider type
    pub _type: ColliderType,
}

impl Collider {
    // Create a new collider
    pub fn new(_type: ColliderType) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            restitution: 0.3,
            friction: ColliderBuilder::default_friction(),
            _type,
        }
    }
    // With
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }
}

// Collider type
pub enum ColliderType {
    Shape(ShapeType),
    Mesh(ObjectID<Mesh>),
}
