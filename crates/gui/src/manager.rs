use crate::painter::Painter;
use rendering::pipeline::Pipeline;
use std::sync::{Arc, Mutex};

// A simple manager
pub struct GUIManager {
    pub egui: egui::CtxRef,
    pub state: egui_winit::State,
    pub painter: Arc<Mutex<Painter>>,
}

impl GUIManager {
    // Create a new GUI manager
    pub fn new(pipeline: &Pipeline) -> Self {
        Self {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(1.0),
            painter: Arc::new(Mutex::new(Painter::new(pipeline))),
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
        egui::Window::new("Test").show(&self.egui, |ui| {});
    }
    // End frame
    pub fn end_frame(&mut self) {
        let (output, clipped_shapes) = self.egui.end_frame();
        let mut painter = self.painter.lock().unwrap();
        let meshes = self.egui.tessellate(clipped_shapes);

        // Set the values using the arc
        painter.font_image = self.egui.font_image().clone();
        painter.clipped_meshes = meshes;
        painter.output = output;
    }
}
