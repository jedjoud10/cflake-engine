use ecs::Component;
use utils::Handle;
use crate::PhysicsSurface;

// Sphere colliders represent perfect spheres in 3D space
// The position of the sphere will be fetched from it's Position component
#[derive(Component)]
pub struct SphereCollider {
    pub radius: f32,
    pub mass: f32,
    pub material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl Clone for SphereCollider {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius.clone(),
            mass: self.mass.clone(),
            material: self.material.clone(),
            sensor: self.sensor,
            handle: None,
        }
    }
} 

impl SphereCollider {
    // Create a new sphere collider with a specific radius and mass
    pub fn new(radius: f32, mass: f32, sensor: bool, material: Option<Handle<PhysicsSurface>>) -> Self {
        Self {
            radius,
            mass,
            sensor,
            handle: None,
            material,
        }
    }
}