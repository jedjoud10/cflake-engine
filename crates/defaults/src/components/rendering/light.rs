use world::ecs::component::Component;

use world::rendering::basics::lights::{LightParameters, LightTransform, LightType};
use world::rendering::pipeline::Handle;
// A light component
#[derive(Component)]
pub struct Light {
    pub light: LightType,
}
