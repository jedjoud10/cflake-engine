use ecs::Component;
use utils::Handle;
use crate::PhysicsSurface;

// Capsule colliders represent capsules with a radius and a height in 3D space
// The position and rotation of the capsule will be fetched from it's Position and Rotation components
#[derive(Component)]
pub struct CapsuleCollider {
    pub(crate) radius: f32,
    pub(crate) height: f32,
    pub(crate) mass: f32,
    pub(crate) material: Option<Handle<PhysicsSurface>>,
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


// Builder for creating a capsule collider
pub struct CapsuleColliderBuilder {
    inner: CapsuleCollider,
}

impl CapsuleColliderBuilder {
    // Create a new capsule collider builder
    pub fn new(mass: f32, radius: f32, height: f32) -> Self {
        Self {
            inner: CapsuleCollider {
                radius,
                height,
                mass,
                material: None,
                sensor: false,
                handle: None,
            },
        }
    }

    // Set the mass of the collider
    pub fn set_mass(mut self, mass: f32) -> Self {
        self.inner.mass = mass;
        self
    }

    // Set the sensor toggle mode of the collider
    pub fn set_sensor(mut self, sensor: bool) -> Self {
        self.inner.sensor = sensor;
        self
    }

    // Set the physics surface material of the collider 
    pub fn set_physics_material(mut self, material: Handle<PhysicsSurface>) -> Self {
        self.inner.material = Some(material);
        self
    }

    // Build the collider
    pub fn build(self) -> CapsuleCollider {
        self.inner
    }
}