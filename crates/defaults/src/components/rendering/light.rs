use world::ecs::component::Component;
use world::rendering::basics::lights::LightSourceType;
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
// A light component
#[derive(Component)]
pub struct Light {
    pub _type: LightSourceType,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            _type: LightSourceType::Directional,
            strength: 1.0,
            color: Default::default(),
        }
    }
}
