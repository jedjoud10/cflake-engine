use world::{
    ecs::component::Component,
    math::shapes::{Cuboid, ShapeType, Sphere},
    physics::rapier3d::prelude::{ColliderBuilder, ColliderHandle},
    rendering::{basics::mesh::Mesh, pipeline::Handle},
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
    // Create a new collider with a specific collider type
    pub fn new(_type: ColliderType) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            restitution: 0.0,
            friction: ColliderBuilder::default_friction(),
            _type,
        }
    }

    // Create a new collider with specific shapes
    pub fn cuboid(size: veclib::Vector3<f32>) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            restitution: 0.0,
            friction: ColliderBuilder::default_friction(),
            _type: ColliderType::Shape(ShapeType::Cuboid(Cuboid {
                center: veclib::Vector3::ZERO,
                size,
            })),
        }
    }
    pub fn sphere(radius: f32) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            restitution: 0.0,
            friction: ColliderBuilder::default_friction(),
            _type: ColliderType::Shape(ShapeType::Sphere(Sphere {
                center: veclib::Vector3::ZERO,
                radius,
            })),
        }
    }
    pub fn mesh(mesh: Handle<Mesh>) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            restitution: 0.0,
            friction: ColliderBuilder::default_friction(),
            _type: ColliderType::Mesh(mesh),
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
    Mesh(Handle<Mesh>),
}
