use std::cell::Cell;

use ecs::Component;
use utils::Handle;
use crate::PhysicsSurface;

// Sphere colliders represent perfect spheres in 3D space
// The position of the sphere will be fetched from it's Position component
#[derive(Component)]
pub struct SphereCollider {
    pub(crate) radius: f32,
    pub(crate) mass: f32,
    pub(crate) material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) modified: Cell<bool>,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl SphereCollider {
    // Update the radius of the sphere collider
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.modified.set(true);
    }

    // Update the mass of the sphere collider
    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
        self.modified.set(true);
    }
    
    // Update the material used by the collider
    pub fn set_material(&mut self, material: Option<Handle<PhysicsSurface>>) {
        self.material = material;
        self.modified.set(true);
    }
    
    // Update the sensor state of the collider
    pub fn set_sensor(&mut self, sensor: bool) {
        self.sensor = sensor;
        self.modified.set(true);
    }
}

impl Clone for SphereCollider {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius.clone(),
            mass: self.mass.clone(),
            modified: Cell::new(false),
            material: self.material.clone(),
            sensor: self.sensor,
            handle: None,
        }
    }
} 

// Builder for creating a cuboid collider
pub struct SphereColliderBuilder {
    inner: SphereCollider,
}

impl SphereColliderBuilder {
    // Create a new cuboid collider builder
    pub fn new(mass: f32, radius: f32) -> Self {
        Self {
            inner: SphereCollider {
                radius,
                mass,
                material: None,
                modified: Cell::new(false),
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
    pub fn build(self) -> SphereCollider {
        self.inner
    }
}