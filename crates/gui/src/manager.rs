use crate::painter::Painter;
use parking_lot::Mutex;
use rendering::pipeline::Pipeline;
use std::sync::Arc;

// A simple manager
pub struct GUIManager {
    pub egui: egui::CtxRef,
    pub state: egui_winit::State,
    pub painter: Painter,
}

impl GUIManager {
    // Create a new GUI manager
    pub fn new(pipeline: &mut Pipeline) -> Self {
        Self {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(1.0),
            painter: Painter::new(pipeline),
        }
    }
    // We have received some events from glutin
    pub fn receive_event(&mut self, event: &egui_winit::winit::event::WindowEvent<'_>) {
        let context = &*self.egui;
        self.state.on_event(context, event);
    }
    // Begin frame
    pub fn begin_frame(&mut self, window: &egui_winit::winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.egui.begin_frame(raw_input);
    }
    // End frame
    pub fn end_frame(&mut self) {
        let (output, clipped_shapes) = self.egui.end_frame();
        let meshes = self.egui.tessellate(clipped_shapes);

        // Draw

        /*
        // Set the values using the arc
        painter.clipped_meshes = meshes;
        painter.output = output;
        painter.egui_font_image_arc = self.egui.font_image();
        */
    }
}
