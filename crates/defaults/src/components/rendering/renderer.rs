use world::ecs::component::Component;
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Handle;
use bitflags::bitflags;
// Renderer flags
bitflags! {
    pub struct RendererFlags: u8 {
        const VISIBLE = 1;
        const SHADOWED = 1 << 1;
    }
}

// A renderer component
#[derive(Component)]
pub struct Renderer {
    // Required for rendering
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,

    // Keep the model matrix cached
    pub matrix: vek::Mat4<f32>,

    // Some rendering settings
    pub flags: RendererFlags,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            matrix: Default::default(),
            flags: RendererFlags::all()
        }
    }
}
