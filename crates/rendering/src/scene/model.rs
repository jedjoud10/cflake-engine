use ecs::Component;
use math::AABB;

// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Model {
    // Model matrix (translation, rotation, scale) that defines this renderer
    matrix: vek::Mat4<f32>,

    // Will the renderer's surfaces be rasterized?
    visible: bool,
    
    // The current AABB bounds that this renderer will use for culling
    bounds: AABB,
}

impl Model {
    // Get the current mesh matrix that we will use for rendering
    pub fn matrix(&self) -> &vek::Mat4<f32> {
        &self.matrix
    }

    // Get the current visibility state of the renderer
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

/*
impl From<math::Transform> for Renderer {
    fn from(_: math::Transform) -> Self {
        Self { matrix: transform.matrix(), visible: true, bounds: math::AABB::default() }
    }
}
*/