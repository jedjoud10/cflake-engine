use crate::painter::Painter;
use rendering::pipeline::{pipec, PipelineContext};
use std::sync::{Arc, Mutex};

// A simple manager
pub struct GUIManager {
    pub egui: egui::CtxRef,
    pub state: egui_winit::State,

    // This is an arc, but it should only be called on the render thread for rendering
    pub painter: Arc<Mutex<Painter>>,
}

impl GUIManager {
    // Create a new GUI manager
    pub fn new(context: &PipelineContext) -> Self {
        // Create an empty arc, and construct the painter on the render thread
        let arc: Arc<Mutex<Option<Painter>>> = Arc::new(Mutex::new(None));
        let cloned = arc.clone();
        pipec::update_callback(&context.read(), move |pipeline, _| {
            // Init on the render thread
            *cloned.lock().unwrap() = Some(Painter::new(pipeline));
        });
        // Flush
        pipec::flush_and_execute(context);

        // Extract
        let painter = if let Ok(ok) = Arc::try_unwrap(arc) {
            ok.into_inner().unwrap().unwrap()
        } else {
            panic!()
        };
        Self {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(1.0),
            painter: Arc::new(Mutex::new(painter)),
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
        let mut painter = self.painter.lock().unwrap();
        let meshes = self.egui.tessellate(clipped_shapes);
        // Set the values using the arc
        painter.clipped_meshes = meshes;
        painter.output = output;
        painter.egui_font_image_arc = self.egui.font_image();
    }
}
