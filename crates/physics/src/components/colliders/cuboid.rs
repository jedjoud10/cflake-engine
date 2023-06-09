use ecs::Component;
use utils::Handle;

use crate::PhysicsSurface;

// Cuboid colliders represent a cuboid in 3D space
// The position and rotation of the cuboid will be fetched from it's Position component and Rotation component
#[derive(Component)]
pub struct CuboidCollider {
    pub half_extent: vek::Extent3<f32>,
    pub mass: f32,
    pub material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl Clone for CuboidCollider {
    fn clone(&self) -> Self {
        Self {
            half_extent: self.half_extent,
            mass: self.mass,
            handle: None,
            sensor: self.sensor,
            material: self.material.clone()
        }
    }
} 

impl CuboidCollider {
    // Create a new cuboid collider with a specific half-extent and mass
    pub fn new(half_extent: vek::Extent3<f32>, mass: f32, sensor: bool, material: Option<Handle<PhysicsSurface>>,) -> Self {
        Self {
            half_extent,
            mass,
            material,
            sensor,
            handle: None,
        }
    }
}