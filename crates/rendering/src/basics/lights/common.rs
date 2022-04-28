use enum_as_inner::EnumAsInner;

// Light source parameters
pub struct LightParameters {
    pub color: vek::Rgb<f32>,
}

impl Default for LightParameters {
    fn default() -> Self {
        Self {
            color: vek::Rgb::one(),
        }
    }
}

// Light transform
pub struct LightTransform<'object> {
    pub position: &'object vek::Vec3<f32>,
    pub rotation: &'object vek::Quaternion<f32>,
}

// A light type
#[derive(EnumAsInner)]
pub enum LightType {
    // Directional light, like the sun
    Directional { params: LightParameters },
    // Point light, like a lamp
    // TODO: Add support for point lights in shader
    Point { params: LightParameters, radius: f32 },
}

impl LightType {
    // Create a new directional light
    pub fn directional(color: vek::Rgb<f32>) -> Self {
        Self::Directional {
            params: LightParameters { color },
        }
    }
    // Create a new point light
    pub fn point(radius: f32, color: vek::Rgb<f32>) -> Self {
        Self::Point {
            params: LightParameters { color },
            radius,
        }
    }
}
