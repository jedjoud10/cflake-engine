use crate::engine::core::ecs::*;

// A component that will be linked to entities that could be ticked
pub struct TickComponent {
	last_tick_time: f32,
	last_tick_id: u32,
}

// Main traits implemented
impl Component for TickComponent {
    fn as_any(&self) -> &dyn std::any::Any {
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl ComponentID for UpdatableComponent {
	fn get_component_name() -> String {
		String::from("Updatable Component")
	}
}

// The current render state of the entity
pub enum EntityRenderState {
	Visible,
	Invisible,
}

// A component that will be linked to entities that are renderable
pub struct RenderComponent {
	pub render_state: EntityRenderState,
	pub vertex_vao: u16,
}

impl Default for RenderComponent {
	fn default() -> Self {
		Self {
			render_state: EntityRenderState::Visible,
			vertex_vao: 0,
		}
	}
}

// Main traits implemented
impl Component for RenderComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl ComponentID for RenderComponent {
	fn get_component_name() -> String {
		String::from("Renderable Component")
	}
}