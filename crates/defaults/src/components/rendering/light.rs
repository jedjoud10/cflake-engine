use world::ecs::Component;
use world::rendering::basics::lights::LightType;
// A light source component
#[derive(Component)]
pub struct Light(pub LightType);
