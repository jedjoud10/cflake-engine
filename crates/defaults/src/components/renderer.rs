use main::{
    math,
    rendering::{self, object::ObjectID},
};
// An Renderer component
pub struct Renderer {
    pub object_id: ObjectID<rendering::basics::renderer::Renderer>,
}

impl Renderer {
    pub fn new(renderer_id: ObjectID<rendering::basics::renderer::Renderer>) -> Self {
        Self { object_id: renderer_id }
    }
}

// Main traits implemented
main::ecs::impl_component!(Renderer);
