use hypo_ecs::*;

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    pub sky_gradient_texture_id: u16,
}

// Main traits implemented
impl ComponentInternal for Sky {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for Sky {
    fn get_component_name() -> String {
        String::from("Sky")
    }
}
impl Component for Sky {}
