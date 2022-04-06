use world::ecs::Component;

use world::rendering::basics::lights::LightType;

// A light component
#[derive(Component)]
pub struct Light {
    pub light: LightType,
}
