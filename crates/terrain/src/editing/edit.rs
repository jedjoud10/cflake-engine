use math::{
    shapes::{Cuboid, ShapeType, Sphere},
};

// Edit parameters
#[derive(Clone)]
pub struct EditParams {
    pub material: Option<u8>,
    pub color: vek::Rgb<u8>,
    pub _union: bool,
}

impl Default for EditParams {
    fn default() -> Self {
        Self { 
            material: Default::default(),
            color: vek::Rgb::one() * 255,
            _union: Default::default()
        }
    }
}

// A single terrain edit
#[derive(Clone)]
pub struct Edit {
    // Contains the shape of the edit and some other edit parameters
    pub shape: ShapeType,

    // Params
    pub params: EditParams,
}

impl Edit {
    // Create a new edit
    pub fn new(shape: ShapeType, params: EditParams) -> Self {
        Self {
            shape,
            params,
        }
    }
    // Create a new sphere edit
    pub fn sphere(center: vek::Vec3<f32>, radius: f32, params: EditParams) -> Self {
        Self::new(ShapeType::Sphere(Sphere { center, radius }), params)
    }
    // Create a new cuboid edit
    pub fn cuboid(center: vek::Vec3<f32>, size: vek::Vec3<f32>, params: EditParams) -> Self {
        Self::new(ShapeType::Cuboid(Cuboid { center, size }), params)
    }
}
