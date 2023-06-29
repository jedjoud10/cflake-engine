use std::cell::Cell;

use ecs::Component;
use utils::Handle;

use crate::PhysicsSurface;

// Cuboid colliders represent a cuboid in 3D space
// The position and rotation of the cuboid will be fetched from it's Position component and Rotation component
#[derive(Component)]
pub struct CuboidCollider {
    pub(crate) half_extent: vek::Extent3<f32>,
    pub(crate) mass: f32,
    pub(crate) material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
    pub(crate) modified: Cell<bool>,
}

impl CuboidCollider {
    // Update the half-extent of the cuboid collider
    pub fn set_half_extent(&mut self, half_extent: vek::Extent3<f32>) {
        self.half_extent = half_extent;
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

impl Clone for CuboidCollider {
    fn clone(&self) -> Self {
        Self {
            half_extent: self.half_extent,
            mass: self.mass,
            handle: None,
            sensor: self.sensor,
            modified: Cell::new(false),
            material: self.material.clone()
        }
    }
} 

// Builder for creating a cuboid collider
pub struct CuboidColliderBuilder {
    inner: CuboidCollider,
}

impl CuboidColliderBuilder {
    // Create a new cuboid collider builder
    pub fn new(mass: f32, half_extent: vek::Extent3<f32>) -> Self {
        Self {
            inner: CuboidCollider {
                half_extent,
                mass,
                material: None,
                sensor: false,
                modified: Cell::new(false),
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
    pub fn build(self) -> CuboidCollider {
        self.inner
    }
}