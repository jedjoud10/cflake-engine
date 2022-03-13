use enum_as_inner::EnumAsInner;
use world::{
    ecs::component::Component,
    math::shapes::{Cuboid, ShapeType, Sphere},
    rendering::{basics::mesh::Mesh, pipeline::Handle},
};
use rapier3d::prelude::{ColliderBuilder, ColliderHandle};
pub use rapier3d::prelude::ColliderMaterial;

// Collider component
#[derive(Component)]
pub struct Collider {
    // Main
    pub(crate) handle: ColliderHandle,
    pub material: ColliderMaterial,
    pub geometry: ColliderGeometry,
}

impl Collider {
    // Create a new collider
    pub fn new(geometry: ColliderGeometry, material: ColliderMaterial) -> Self {
        Self {
            handle: ColliderHandle::invalid(),
            material,
            geometry,
        }
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self { 
            handle: ColliderHandle::invalid(),
            material: ColliderMaterial::default(),
            geometry: ColliderGeometry::Shape(ShapeType::Cuboid(Cuboid::default()))
        }
    }
}

// Collider type
#[derive(EnumAsInner, Clone)]
pub enum ColliderGeometry {
    Shape(ShapeType),
    Mesh {
        mesh: Handle<Mesh>,
        mass: f32,
        com_offset: vek::Vec3<f32>
    },
}

impl ColliderGeometry {
    // Create a new collider with specific shapes
    pub fn cuboid(size: vek::Vec3<f32>) -> Self {
        ColliderGeometry::Shape(ShapeType::Cuboid(Cuboid {
            center: vek::Vec3::ZERO,
            size,
        }))
    }
    pub fn sphere(radius: f32) -> Self {
        ColliderGeometry::Shape(ShapeType::Sphere(Sphere {
            center: vek::Vec3::ZERO,
            radius,
        }))
    }
    pub fn mesh(mesh: Handle<Mesh>, mass: f32) -> Self {
        ColliderGeometry::Mesh {
            mesh,
            mass,
            com_offset: vek::Vec3::ZERO,
        }
    }
}