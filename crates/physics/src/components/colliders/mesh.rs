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
    pub(crate) handle: Option<rapier3d::geometry::ColliderHandle>,
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