use world::ecs::component::Component;
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
use world::rendering::basics::uniforms::StoredUniforms;
use world::rendering::pipeline::Handle;
// A renderer component
#[derive(Component)]
pub struct Renderer {
    // Required for rendering
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,

    // Keep the model matrix cached
    pub matrix: veclib::Matrix4x4<f32>,

    // Some rendering settings
    pub visible: bool,
    pub shadowed: bool,
    pub uniforms: StoredUniforms,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            matrix: Default::default(),
            visible: true,
            shadowed: true,
            uniforms: Default::default(),
        }
    }
}
