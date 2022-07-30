use ecs::Component;

// Main color type
type Color = vek::Rgb<u8>;

// A directional light is a type of light that emits light in a single direction (like the sun)
#[derive(Component)]
pub struct DirectionalLight {
    // The color of the light
    pub color: Color,

    // The strength the light
    pub strength: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Color::broadcast(255),
            strength: 1.0,
        }
    }
}

// A point light is a type of light that emits light in all direction, coming from a single point (depicted from the Transform of this entity)
#[derive(Component)]
pub struct PointLight {
    // The color of the light
    pub color: Color,

    // The strength of the light (in lumens or lux)
    pub strength: f32,

    // The sphere of influence of the light
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color::broadcast(255),
            strength: 1.0,
            radius: 10.0,
        }
    }
}
