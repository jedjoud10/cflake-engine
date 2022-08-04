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
#[repr(C)]
pub struct PointLight {
    // The color of the light
    pub color: vek::Rgb<u8>,

    // The strength of the light (in lumens or lux)
    pub strength: f32,

    // The sphere of influence of the light
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: vek::Rgb::broadcast(255),
            strength: 1.0,
            radius: 10.0,
        }
    }
}
