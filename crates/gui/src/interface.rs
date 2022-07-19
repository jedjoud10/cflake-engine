use crate::painter::Painter;
use assets::Assets;
use egui_winit::winit::event::WindowEvent;
use rendering::{
    gl,
    prelude::{Context, Window},
};
use world::{Events, Resource, World};

// This interface encapsulates all the data that we need to use eGui and to draw
// There are no functions associated with the struct, since everything is handled from within the system alreadyz
pub struct UserInterface {
    // This is the egui context that will handle everything related to egui
    egui: egui::Context,

    // This is the current egui state given from glutin whenever we receive a new window event
    state: egui_winit::State,

    // This is the painter that will actually draw stuff onto the screen
    painter: Painter,
}

impl AsMut<egui::Context> for UserInterface {
    fn as_mut(&mut self) -> &mut egui::Context {
        &mut self.egui
    }
}

impl AsRef<egui::Context> for UserInterface {
    fn as_ref(&self) -> &egui::Context {
        &self.egui
    }
}

// This system will automatically insert the user interface and setup it's necessary events
// This will create the init event, begin update event, draw update event, and window event
pub fn system(_events: &mut Events) {
    /*/
    // Create a new GUI manager using an asset loader and OpenGL context at the start of the program
    fn init(world: &mut World) {
        let (ctx, assets) = world.get_mut::<(&mut Context, &mut Assets)>().unwrap();

        // Get the maximum texture size from OpenGL (idk why egui needs this tbh)
        let max_texture_size = unsafe {
            let mut max: i32 = 0;
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max);
            max as usize
        };

        // Construct the user interface and add it as a resource
        let ui = UserInterface {
            egui: Default::default(),
            state: egui_winit::State::from_pixels_per_point(max_texture_size, 1.0),
            painter: Painter::new(assets, ctx),
        };
        world.insert(ui);
    }

    // Called from within glutin to register a new window event
    fn window(world: &mut World, event: &WindowEvent) {
        let ui = world.get_mut::<&mut UserInterface>().unwrap();
        ui.state.on_event(&mut ui.egui, event);
    }

    // This is called at the start of each frame to tell egui that we must register the upcoming draw commands
    fn begin(world: &mut World) {
        let (ui, window) = world
            .get_mut::<(&mut UserInterface, &mut Window)>()
            .unwrap();
        let raw_input = ui.state.take_egui_input(window.raw());
        ui.egui.begin_frame(raw_input);
    }

    // This is called at the end of each frame (after we render the main 3D scene)
    fn draw(world: &mut World) {
        let (ui, window, ctx, assets) = world
            .get_mut::<(&mut UserInterface, &mut Window, &mut Context, &mut Assets)>()
            .unwrap();

        let output = ui.egui.end_frame();
        ui.state
            .handle_platform_output(window.raw(), &mut ui.egui, output.platform_output);

        let clipped_shapes = output.shapes;
        let deltas = output.textures_delta;
        let meshes = ui.egui.tessellate(clipped_shapes);

        if !meshes.is_empty() {
            ui.painter.draw(window, ctx, meshes, assets, deltas);
        }
    }

    // Register all the events
    /*
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(begin);
    events
        .registry::<Update>()
        .insert_with(
            draw,
            Stage::new("ui rendering")
                .after("scene rendering")
                .before("window back buffer swap"),
        )
        .unwrap();
    events.registry::<WindowEvent>().insert(window);
    */
    */
}
