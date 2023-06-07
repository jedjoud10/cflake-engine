use ecs::Component;

// A directional light is a type of light that emits light in a single direction (like the sun)
#[derive(Component, Clone, Copy)]
pub struct DirectionalLight {
    // Intensity of the directional light
    // TODO: Get units
    pub intensity: f32,

    // RGB8 color of the light
    pub color: vek::Rgb<u8>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            intensity: 4.0,
            color: vek::Rgb::broadcast(255),
        }
    }
}

// A point light which emits light in all directions
#[derive(Component, Clone, Copy)]
pub struct PointLight {
    // Intensity of the point light
    // TODO: Get units
    pub intensity: f32,
    
    // Max light distance used for attenuation calculations
    pub radius: f32,
    
    // RGB8 color of the light
    pub color: vek::Rgb<u8>,
}