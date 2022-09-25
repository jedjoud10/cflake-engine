use ecs::Component;
// A directional light is a type of light that emits light in a single direction (like the sun)
#[derive(Component, Clone, Copy)]
#[repr(C)]
pub struct DirectionalLight {
    // The color of the light
    pub color: vek::Rgb<u8>,

    // The strength the light
    pub strength: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: vek::Rgb::broadcast(255),
            strength: 1.0,
        }
    }
}

// A point light is a type of light that emits light in all direction, coming from a single point (depicted from the Transform of this entity)
#[derive(Component, Clone, Copy)]
pub struct PointLight {
    pub color: vek::Rgb<u8>,
    pub strength: f32,
    pub attenuation: f32,
}

// The packed light data that will be used within the compute buffer
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct PackedPointLight {
    pub color: vek::Rgba<f32>,
    pub position_attenuation: vek::Vec4<f32>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: vek::Rgb::broadcast(255),
            strength: 9.0,
            attenuation: 0.5,
        }
    }
}
