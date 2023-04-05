use ecs::Component;

// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Renderer {
    // Model matrix (translation, rotation, scale) that defines this renderer
    pub matrix: vek::Mat4<f32>,

    // Is the model currently enabled for rendering (this ignores if the model is culled or not)
    pub visible: bool,

    // Contains the frame timings the surface was created
    pub instant_initialized: Option<std::time::Instant>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            matrix: vek::Mat4::identity(),
            visible: true,
            instant_initialized: Some(std::time::Instant::now()),
        }
    }
}
