use main::rendering::{self, object::ObjectID};
// An Renderer component
pub struct Renderer {
    // The CPU renderer that we will store until we send the construction task
    pub(crate) renderer: Option<rendering::basics::renderer::Renderer>,

    // The returned Object ID for our Renderer that is stored on the GPU Pipeline
    pub(crate) object_id: ObjectID<rendering::basics::renderer::Renderer>,
}

impl Renderer {
    // Create a new renderer component using a CPU renderer object
    pub fn new(renderer: rendering::basics::renderer::Renderer) -> Self {
        Self { renderer: Some(renderer), object_id: ObjectID::default() }
    }
}

// Main traits implemented
main::ecs::impl_component!(Renderer);
