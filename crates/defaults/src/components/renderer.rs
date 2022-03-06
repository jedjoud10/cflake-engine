use world::ecs::component::Component;
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Handle;
// A renderer component
#[derive(Component)]
pub struct Renderer {
    // Required for rendering
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,

    // Some rendering settings
    pub visible: bool,
    pub shadow_caster: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            visible: true,
            shadow_caster: true,
        }
    }
}
