use ecs::Component;
use utils::Handle;
use crate::PhysicsSurface;

// Capsule colliders represent capsules with a radius and a height in 3D space
// The position and rotation of the capsule will be fetched from it's Position and Rotation components
#[derive(Component)]
pub struct CapsuleCollider {
    pub radius: f32,
    pub height: f32,
    pub mass: f32,
    pub material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl Clone for CapsuleCollider {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius,
            height: self.height,
            mass: self.mass,
            sensor: self.sensor,
            material: self.material.clone(),
            handle: None,
        }
    }
} 

impl CapsuleCollider {
    pub fn new(radius: f32, height: f32, mass: f32, sensor: bool, material: Option<Handle<PhysicsSurface>>) -> Self {
        Self {
            radius,
            height,
            mass,
            material,
            sensor,
            handle: None,
        }
    }
}