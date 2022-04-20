use enum_as_inner::EnumAsInner;
use rapier3d::prelude::ColliderHandle;
pub use rapier3d::prelude::ColliderMaterial;
use world::{
    ecs::Component,
    math::shapes::{Cuboid, ShapeType, Sphere},
    rendering::{basics::mesh::Mesh, pipeline::Handle},
};

// Collider component
#[derive(Component)]
pub struct Collider {
    // Rapier's collider handle
    pub(crate) handle: ColliderHandle,

    // Our built-in collider material that handles friction and restitution
    pub material: ColliderMaterial,

    // The collider's geometry; The actual shape of the collider
    pub geometry: ColliderGeometry,
}

impl Collider {
    // Create a new collider from collider geometry
    pub fn new(geometry: ColliderGeometry, material: ColliderMaterial) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            material,
            geometry,
        }
    }
}

// Simple collider builder
pub struct ColliderBuilder(Collider);

impl ColliderBuilder {
    // Cuboid shape, default material
    pub fn cuboid(size: vek::Vec3<f32>) -> Self {
        Self(Collider::new(
            ColliderGeometry::Shape(ShapeType::Cuboid(Cuboid { center: vek::Vec3::zero(), size })),
            Default::default(),
        ))
    }
    // Sphere shape, default material
    pub fn sphere(radius: f32) -> Self {
        Self(Collider::new(
            ColliderGeometry::Shape(ShapeType::Sphere(Sphere {
                center: vek::Vec3::zero(),
                radius,
            })),
            Default::default(),
        ))
    }
    // Mesh "shape", default material
    pub fn mesh(mesh: Handle<Mesh>, mass: f32, com: vek::Vec3<f32>) -> Self {
        Self(Collider::new(ColliderGeometry::Mesh { mesh, mass, com }, Default::default()))
    }

    // Update the collider's default material to something fancier
    pub fn with_material(mut self, mat: ColliderMaterial) -> Self {
        self.0.material = mat;
        self
    }

    // Build the collider
    pub fn build(self) -> Collider {
        self.0
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            material: ColliderMaterial::default(),
            geometry: ColliderGeometry::Shape(ShapeType::Cuboid(Cuboid::default())),
        }
    }
}

// Collider type
#[derive(EnumAsInner, Clone)]
pub enum ColliderGeometry {
    Shape(ShapeType),
    Mesh { mesh: Handle<Mesh>, mass: f32, com: vek::Vec3<f32> },
}
