use assets::Assets;
use egui_winit::winit::event_loop::EventLoop;
use graphics::{Graphics, Window};
use world::{World, System, post_user, WindowEvent};
use crate::Interface;

// Insert the required Egui resources and the render pass
fn init(world: &mut World, el: &EventLoop<()>) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let window = world.get::<Window>().unwrap();

    // Construct the user interface and add it as a resource
    let ui = Interface {
        egui: Default::default(),
        state: egui_winit::State::new(el),
    };

    // TODO: Pls remove. It's kinda getting annoying now tbh
    drop(graphics);
    drop(assets);
    drop(window);
    
    // Insert the resource into the world
    world.insert(ui);
}

// Called from within winit to register a new window event
fn event(world: &mut World, event: &mut WindowEvent) {
    let mut interface = world.get_mut::<Interface>().unwrap();
    let ui    = &mut *interface;
    todo!()
    //ui.state.on_event(&mut ui.egui, event);
}


// Render the egui meshes to the current window texture using the render pass
fn render(world: &mut World) {
}

// Add the egui resources, and render to proper target texture
pub fn system(system: &mut System) {
    system.insert_init(init)
        .after(graphics::common)
        .before(post_user);
    system.insert_update(render)
        .after(rendering::systems::rendering::system)
        .before(graphics::present)
        .before(rendering::systems::composite::system);
    system.insert_window(event)
        .before(graphics::present)
        .before(graphics::acquire)
        .after(graphics::common);
}

/*


// This system will automatically insert the user interface and setup it's necessary events
// This will create the init event, begin update event, draw update event, and window event
pub fn system(events: &mut Events) {
    // Create a new GUI manager using an asset loader and OpenGL context at the start of the program
    fn init(world: &mut World) {
        
    }
    // This is called at the start of each frame to tell egui that we must register the upcoming draw commands
    fn begin(world: &mut World) {
        let mut ui = world.get_mut::<UserInterface>().unwrap();
        let window = world.get_mut::<Window>().unwrap();
        let raw_input = ui.state.take_egui_input(window.raw());
        ui.egui.begin_frame(raw_input);
    }

    // This is called at the end of each frame (after we render the main 3D scene)
    fn draw(world: &mut World) {
        let mut interface = world.get_mut::<UserInterface>().unwrap();
        let ui = &mut *interface;
        let mut window = world.get_mut::<Window>().unwrap();
        let mut ctx = world.get_mut::<Context>().unwrap();
        let mut assets = world.get_mut::<Assets>().unwrap();

        let output = ui.egui.end_frame();
        ui.state
            .handle_platform_output(window.raw(), &ui.egui, output.platform_output);

        let clipped_shapes = output.shapes;
        let deltas = output.textures_delta;
        let meshes = ui.egui.tessellate(clipped_shapes);

        if !meshes.is_empty() {
            ui.painter
                .draw(&mut window, &mut ctx, meshes, &mut assets, deltas);
        }
    }

    // Register all the events
    events
        .registry::<Init>()
        .insert_with(init, Stage::new("ui insert").after("graphics insert"))
        .unwrap();
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
}

 */