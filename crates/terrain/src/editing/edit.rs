use math::{
    csg::CSGOperation,
    shapes::{Cuboid, ShapeType, Sphere},
};

// A single terrain edit
#[derive(Clone)]
pub struct Edit {
    // Contains the shape of the edit and some other edit parameters
    pub shape: ShapeType,

    // Params
    pub material: Option<u8>,
    pub color: vek::Vec3<u8>,
    pub operation: CSGOperation,
}

impl Default for Edit {
    fn default() -> Self {
        Self {
            shape: ShapeType::Sphere(Sphere {
                center: vek::Vec3::zero(),
                radius: 10.0,
            }),
            material: Default::default(),
            color: vek::Vec3::one() * 255,
            operation: CSGOperation::Union,
        }
    }
}

impl Edit {
    // Create a new edit
    pub fn new(shape: ShapeType, operation: CSGOperation) -> Self {
        Self {
            shape,
            material: None,
            color: vek::Vec3::one() * 255,
            operation,
        }
    }
    // Create a new sphere edit
    pub fn sphere(center: vek::Vec3<f32>, radius: f32, operation: CSGOperation, material: Option<u8>) -> Self {
        Self {
            shape: ShapeType::Sphere(Sphere { center, radius }),
            operation,
            material,
            ..Default::default()
        }
    }
    // Create a new cuboid edit
    pub fn cuboid(center: vek::Vec3<f32>, size: vek::Vec3<f32>, operation: CSGOperation, material: Option<u8>) -> Self {
        Self {
            shape: ShapeType::Cuboid(Cuboid { center, size }),
            operation,
            material,
            ..Default::default()
        }
    }
}
