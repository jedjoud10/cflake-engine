use crate::pack_color;
use math::shapes::{Cuboid, ShapeType, Sphere};

// Edit parameters that depict how the edit will influence the terrain
#[derive(Clone)]
pub struct EditParams {
    material: Option<u8>,
    color: vek::Rgb<u8>,
    _union: bool,
}

impl EditParams {
    pub fn new(material: Option<u8>, color: vek::Rgb<f32>, union: bool) -> Self {
        Self {
            material,
            color: (color * 255.0).as_(),
            _union: union,
        }
    }

    // Convert the parameters into a single u32 that can be sent to the GPU
    pub fn convert(&self, shapetype: u8) -> u32 {
        let rgbcolor = (pack_color(self.color) as u32) << 16; // 2
        let shape_type_edit_type = (((shapetype << 4) | (!self._union as u8)) as u32) << 8; // 1
        let material = self.material.unwrap_or(255) as u32; // 1
        let rgbcolor_shape_type_edit_type_material = rgbcolor | shape_type_edit_type | material;
        rgbcolor_shape_type_edit_type_material
    }
}

impl Default for EditParams {
    fn default() -> Self {
        Self {
            material: Default::default(),
            color: vek::Rgb::one() * 255,
            _union: true,
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
        Self { shape, params }
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
