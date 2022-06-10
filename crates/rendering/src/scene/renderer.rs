use ecs::Component;

// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Renderer {
    matrix: vek::Mat4<f32>,
    visible: bool,
    //bounds: math::AABB,
}