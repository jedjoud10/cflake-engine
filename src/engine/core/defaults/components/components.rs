use crate::engine::core::ecs::*;

// A component that will be linked to entities that could be ticked
pub struct TickComponent {
	last_tick_time: f32,
	last_tick_id: u32,
}

// Main traits implemented
impl Component for TickComponent {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for TickComponent {
	fn get_component_name() -> String {
		String::from("Tick Component")
	}
}


// A component that will be linked to entities that could be update each single frame
pub struct UpdatableComponent {
	priority: u16
}

// Main traits implemented
impl Component for UpdatableComponent {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for UpdatableComponent {
	fn get_component_name() -> String {
		String::from("Updatable Component")
	}
}