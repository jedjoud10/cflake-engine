use bitflags::bitflags;
use world::ecs::Component;
use world::math::bounds::aabb::AABB;
use world::rendering::basics::material::Material;
use world::rendering::basics::mesh::Mesh;
use world::rendering::pipeline::Handle;
// Renderer flags
bitflags! {
    pub struct RendererFlags: u8 {
        const VISIBLE = 1;
        const SHADOW_CASTER = 1 << 1;
        const MATRIX_UPDATE = 1 << 2;
    }
}

// A renderer component
#[derive(Component)]
pub struct Renderer {
    // Required for rendering
    pub mesh: Handle<Mesh>,
    pub material: Handle<Material>,
    pub(crate) bounds: AABB,

    // Keep the model matrix cached
    pub(crate) matrix: vek::Mat4<f32>,

    // Some rendering settings
    pub(crate) flags: RendererFlags,
}

impl From<Handle<Mesh>> for Renderer {
    fn from(mesh: Handle<Mesh>) -> Self {
        Self { mesh, ..Default::default() }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            matrix: Default::default(),
            flags: RendererFlags::all(),
            bounds: AABB::default(),
        }
    }
}

impl Renderer {
    pub fn new(mesh: Handle<Mesh>, material: Handle<Material>) -> Self {
        Self {
            mesh,
            material,
            ..Default::default()
        }
    }
}
