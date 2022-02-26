use math::shapes::{Cuboid, Sphere};

use crate::{identifier::PhysicsID, surface::Surface};

use super::MeshCollider;

// The collider shape type
pub enum ColliderType {
    Cuboid(Cuboid),
    Sphere(Sphere),
    Mesh(MeshCollider),
}

impl ColliderType {
    // Get the center of the inner collider
    pub fn try_get_center(&self) -> Option<&veclib::Vector3<f32>> {
        match &self {
            ColliderType::Cuboid(cuboid) => Some(&cuboid.center),
            ColliderType::Sphere(sphere) => Some(&sphere.center),
            ColliderType::Mesh(_) => None,
        }
    }
}

// A simple shape collider
pub struct Collider {
    // Surface type
    pub surface: PhysicsID<Surface>,
    // Shape
    pub shape: ColliderType,
}
