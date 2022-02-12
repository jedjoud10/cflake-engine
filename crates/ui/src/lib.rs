// Simply re-export
pub use egui_glfw_gl::{
    egui, EguiInputState
};
// A simple manager
pub struct UIManager {
    // The contexts
    pub ui: egui::CtxRef,
    pub input_state: egui_glfw_gl::EguiInputState,
}

impl UIManager {
    // New
    pub fn new(width: u16, height: u16, pixel_per_point: f32) -> Self {
        use egui_glfw_gl::egui::*;
        Self {
            ui: egui::CtxRef::default(),
            input_state: EguiInputState::new(egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::new(0.0, 0.0),
                    vec2(width as f32, height as f32) / pixel_per_point,
                )),
                pixels_per_point: Some(pixel_per_point),
                ..Default::default()
            }),
        }
    }
    // Update the screen dimensions
    pub fn update_screen_dimensions(&mut self, width: u16, height: u16, pixel_per_point: f32) {
        use egui_glfw_gl::egui::*;
        self.input_state = EguiInputState::new(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::new(0.0, 0.0),
                vec2(width as f32, height as f32) / pixel_per_point,
            )),
            pixels_per_point: Some(pixel_per_point),
            ..Default::default()
        });
    }
    // Handle event
    pub fn handle_event(&mut self, event: egui_glfw_gl::glfw::WindowEvent) {
        egui_glfw_gl::handle_event(event, &mut self.input_state);
    }
}