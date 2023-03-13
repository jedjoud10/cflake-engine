// A directional light is a type of light that emits light in a single direction (like the sun)
#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct DirectionalLight {
    // The strength the light
    pub strength: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self { strength: 1.0 }
    }
}
