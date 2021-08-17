use super::core::world::World;

pub mod model;
pub mod renderer;
pub mod shader;
pub mod texture;

// A window class to organize things
#[derive(Clone)]
pub struct Window {
    pub fullscreen: bool,
    pub size: (u16, u16),
}

impl Default for Window {
	fn default() -> Self {
		Self {
			fullscreen: false,
			size: World::get_default_window_size()
		}
	}
}
