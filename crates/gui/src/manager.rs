use crate::painter::Painter;
use egui_winit::winit::event::WindowEvent;
use rendering::gl;
use rendering::pipeline::{Pipeline, SceneRenderer};

// A simple manager
pub struct GUIManager {
    pub egui: egui::Context,
    pub state: egui_winit::State,
    pub painter: Painter,
}

impl GUIManager {
    // Create a new GUI manager
    pub fn new(pipeline: &mut Pipeline) -> Self {
        let max_texture_size = unsafe {
            let mut max: i32 = 0;
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max);
            max as usize
        };
        Self {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(max_texture_size, 1.0),
            painter: Painter::new(pipeline),
        }
    }
    // We have received some events from glutin
    pub fn receive_event(&mut self, event: &WindowEvent<'_>) -> bool {
        let context = &self.egui;
        self.state.on_event(context, event)
    }
    // Begin frame
    pub fn begin_frame(&mut self, window: &egui_winit::winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.egui.begin_frame(raw_input);
    }
    // End frame
    pub fn draw_frame(&mut self, pipeline: &mut Pipeline, renderer: &mut SceneRenderer) {
        let window = pipeline.window().context().window();
        let output = self.egui.end_frame();
        // Decompose
        self.state.handle_platform_output(window, &mut self.egui, output.platform_output);
        let clipped_shapes = output.shapes;
        let deltas = output.textures_delta;
        let meshes = self.egui.tessellate(clipped_shapes);
        // Draw the GUI
        self.painter.draw_gui(pipeline, renderer, meshes, deltas);
    }
}
