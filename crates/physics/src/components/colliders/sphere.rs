use std::cell::Cell;

use crate::{GenericCollider, PhysicsSurface};
use ecs::{Component, Entity};
use utils::Handle;

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
            radius: self.radius,
            mass: self.mass,
            modified: Cell::new(false),
            material: self.material.clone(),
            sensor: self.sensor,
            handle: None,
        }
    }
}

impl GenericCollider for SphereCollider {
    type RawRapierCollider = rapier3d::geometry::Ball;

    #[inline(always)]
    fn handle(&self) -> Option<rapier3d::geometry::ColliderHandle> {
        self.handle
    }

    #[inline(always)]
    fn set_handle(&mut self, handle: rapier3d::geometry::ColliderHandle) {
        self.handle = Some(handle);
    }

    #[inline(always)]
    fn modified(&self) -> &Cell<bool> {
        &self.modified
    }

    #[inline(always)]
    fn mass(&self) -> f32 {
        self.mass
    }

    #[inline(always)]
    fn material(&self) -> &Option<Handle<PhysicsSurface>> {
        &self.material
    }

    #[inline(always)]
    fn cast_rapier_collider(
        generic: &mut rapier3d::geometry::Collider,
    ) -> &mut Self::RawRapierCollider {
        generic.shape_mut().as_ball_mut().unwrap()
    }

    #[inline(always)]
    fn regenerate_when_updating() -> bool {
        false
    }

    #[inline(always)]
    fn build_collider(&mut self, entity: &Entity) -> Option<rapier3d::geometry::Collider> {
        Some(
            rapier3d::geometry::ColliderBuilder::ball(self.radius)
                .mass(self.mass)
                .sensor(self.sensor)
                .user_data(entity.to_raw() as u128)
                .build(),
        )
    }

    #[inline(always)]
    fn set_custom_rapier_collider_settings(&self, custom: &mut Self::RawRapierCollider) {
        custom.radius = self.radius
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
