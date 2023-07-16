use std::cell::Cell;

use crate::{GenericCollider, PhysicsSurface};
use ecs::{Component, Entity};

use utils::Handle;

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

impl GenericCollider for MeshCollider {
    type RawRapierCollider = rapier3d::geometry::TriMesh;

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
        generic.shape_mut().as_trimesh_mut().unwrap()
    }

    #[inline(always)]
    fn regenerate_when_updating() -> bool {
        true
    }

    #[inline(always)]
    fn build_collider(&mut self, entity: &Entity) -> Option<rapier3d::geometry::Collider> {
        let vertices = self.vertices.take()?;
        let triangles = self.triangles.take()?;

        if vertices.is_empty() || triangles.is_empty() {
            return None;
        }

        let vertices: Vec<rapier3d::na::Point3<f32>> = vertices
            .into_iter()
            .map(crate::vek_vec_to_na_point)
            .collect::<_>();

        Some(
            rapier3d::geometry::ColliderBuilder::trimesh(vertices, triangles)
                .mass(self.mass)
                .sensor(self.sensor)
                .user_data(entity.to_raw() as u128)
                .build(),
        )
    }

    #[inline(always)]
    fn set_custom_rapier_collider_settings(&self, _custom: &mut Self::RawRapierCollider) {}
}

// Builder for creating a mesh collider
pub struct MeshColliderBuilder {
    inner: MeshCollider,
}

impl MeshColliderBuilder {
    // Create a new mesh collider builder
    pub fn new(_vertices: Vec<vek::Vec3<f32>>, _triangles: Vec<[u32; 3]>, mass: f32) -> Self {
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
