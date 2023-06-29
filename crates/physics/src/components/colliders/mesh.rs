use std::cell::Cell;

use ecs::Component;
use rendering::Mesh;
use utils::Handle;
use crate::{PhysicsSurface};

// Mesh collider that will represent a mesh using it's triangles and vertices
#[derive(Component)]
pub struct MeshCollider {
    pub(crate) vertices: Option<Vec<vek::Vec3<f32>>>,
    pub(crate) triangles: Option<Vec<[u32; 3]>>,
    pub(crate) mass: f32,
    pub(crate) material: Option<Handle<PhysicsSurface>>,
    pub(crate) sensor: bool,
    pub(crate) modified: Cell<bool>,
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
}

impl MeshCollider {
    // Update the vertices and triangles used by the mesh collider 
    pub fn set_geometry(&mut self, vertices: Vec<vek::Vec3<f32>>, triangles: Vec<[u32; 3]>) {
        self.vertices = Some(vertices);
        self.triangles = Some(triangles);
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

// Builder for creating a mesh collider
pub struct MeshColliderBuilder {
    inner: MeshCollider,
}

impl MeshColliderBuilder {
    // Create a new mesh collider builder
    pub fn new(vertices: Vec<vek::Vec3<f32>>, triangles: Vec<[u32; 3]>, mass: f32) -> Self {
        Self {
            inner: MeshCollider {
                vertices: None,
                triangles: None,
                mass,
                material: None,
                sensor: false,
                handle: None,
                modified: Cell::new(false),
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
    pub fn build(self) -> MeshCollider {
        self.inner
    }
}