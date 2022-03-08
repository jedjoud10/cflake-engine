use crate::painter::Painter;

use rendering::pipeline::Pipeline;

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
    pub fn draw_frame(&mut self, pipeline: &mut Pipeline) {
        let (output, clipped_shapes) = self.egui.end_frame();
        let meshes = self.egui.tessellate(clipped_shapes);
        self.painter.draw_gui(pipeline, meshes, self.egui.font_image().as_ref(), output);
    }
}
