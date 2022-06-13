use ecs::Component;

type Color = vek::Rgb<u8>;

// A light trait implemented for each type of light
pub trait Light {
    // Get the light's color
    fn color(&self) -> Color;

    // Get the light's strength
    fn strength(&self) -> f32;
}

// A point light is a type of light that emits from a single point (depicted from the Transform of this entity)
#[derive(Component)]
pub struct Point {
    // The color of the light
    color: Color,

    // The strength of the light (in lumens or lux)
    strength: f32,

    // The sphere of influence of the light
    radius: f32
}