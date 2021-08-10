pub mod model;
pub mod renderer;
pub mod shader;
pub mod texture;

// A window class to organize things
#[derive(Default)]
pub struct Window {
	pub fullscreen: bool,
	pub size: (i32, i32),
	pub system_renderer_component_index: u16,
}
