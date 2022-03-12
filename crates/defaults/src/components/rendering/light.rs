use world::ecs::component::Component;

use world::rendering::basics::lights::LightType;

// A light component
#[derive(Component)]
pub struct Light {
    pub light: LightType,
}
