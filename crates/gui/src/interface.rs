use crate::painter::Painter;
use assets::loader::AssetLoader;
use egui_winit::winit::event::WindowEvent;
use rendering::{context::Context, gl};

// A simple manager that will encapsulate everything that is related to GUI
pub struct UserInterface {
    // Main egui related fields
    egui: egui::Context,
    state: egui_winit::State,

    // Custom painter to draw them shits
    painter: Painter,
}

impl UserInterface {
    // Create a new GUI manager using an asset loader and OpenGL context
    pub fn new(loader: &mut AssetLoader, ctx: &mut Context) -> Self {
        // Get the maximum texture size from OpenGL (idk why egui needs this tbh)
        let max_texture_size = unsafe {
            let mut max: i32 = 0;
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max);
            max as usize
        };

        // Construct the manager
        Self {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(max_texture_size, 1.0),
            painter: Painter::new(loader, ctx),
        }
    }

    // Called from within glutin to register a new window event
    pub fn receive_event(&mut self, event: &WindowEvent<'_>) -> bool {
        let context = &self.egui;
        self.state.on_event(context, event)
    }

    // This should be called at the start of each game update frame
    pub fn begin_frame(&mut self, window: &egui_winit::winit::window::Window) {
        let raw_input = self.state.take_egui_input(window);
        self.egui.begin_frame(raw_input);
    }

    // This should be called at the end of each game update frame (after rendering is done)
    pub fn draw_frame(&mut self, ctx: &mut Context) {
        /*
        let window = pipeline.window().context().window();
        let output = self.egui.end_frame();
        // Decompose
        self.state.handle_platform_output(window, &mut self.egui, output.platform_output);
        let clipped_shapes = output.shapes;
        let deltas = output.textures_delta;
        let meshes = self.egui.tessellate(clipped_shapes);
        // Draw the GUI
        self.painter.draw_gui(pipeline, renderer, meshes, deltas);
        */
    }
}