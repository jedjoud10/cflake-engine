use ecs::Component;

// A directional light is a type of light that emits light in a single direction (like the sun)
#[derive(Component, Clone, Copy)]
pub struct DirectionalLight {
    pub color: vek::Rgb<f32>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: vek::Rgb::broadcast(1.0)
        }
    }
}
