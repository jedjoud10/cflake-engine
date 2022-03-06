use world::ecs::component::Component;
use world::rendering::basics::lights::{LightSourceType, StoredLight};
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Handle;
// A light component
#[derive(Component)]
pub struct Light {
    // TODO: Fix duplicate code
    pub(crate) handle: Handle<StoredLight>,

    pub _type: LightSourceType,
    pub strength: f32,
    pub color: veclib::Vector3<f32>,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            _type: LightSourceType::Directional,
            strength: 1.0,
            color: Default::default(),
        }
    }
}
