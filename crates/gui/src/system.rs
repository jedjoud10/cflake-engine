use assets::Assets;
use egui_winit::winit::event_loop::EventLoop;
use graphics::{Graphics, Window};
use world::{World, System, post_user, WindowEvent, user};
use crate::{Interface, Rasterizer};

// Insert the required Egui resources and the render pass
fn init(world: &mut World, el: &EventLoop<()>) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let window = world.get::<Window>().unwrap();

    // Construct the user interface and add it as a resource
    let ui = Interface {
        egui: Default::default(),
        state: egui_winit::State::new(el),
        rasterizer: Rasterizer::new(&graphics, &mut assets),
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
    let interface = &mut *interface;

    // TODO: Handle consumed field
    let _ = interface.state.on_event(&mut interface.egui, event);
}

// Begin the frame by acquiring input and starting the Egui command recorder
fn begin(world: &mut World) {
    let mut interface = world.get_mut::<Interface>().unwrap();
    let window = world.get::<Window>().unwrap();
    let taken = interface.state.take_egui_input(window.raw());
    interface.egui.begin_frame(taken);
}

// Render the egui meshes to the current window texture using the render pass
fn finish(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut interface = world.get_mut::<Interface>().unwrap();
    let ui = &mut *interface;
    let mut window = world.get_mut::<Window>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // End the Egui frame and fetch the meshes
    let output = ui.egui.end_frame();
    ui.state
        .handle_platform_output(window.raw(), &ui.egui, output.platform_output);
    let tessellated = ui.tessellate(output.shapes);
    interface.rasterizer.draw(
        &graphics,
        &mut window,
        &mut assets,
        tessellated,
        output.textures_delta,
    );
}

// Common system wil contain the DeviceEvent and will insert the Egui resources
pub fn common(system: &mut System) {
    system.insert_init(init)
        .after(graphics::common)
        .before(post_user);
    system.insert_window(event)
        .after(post_user)
        .after(graphics::common);
}

// Acquire system will begin recording egui commands at the start of the frame
pub fn acquire(system: &mut System) {
    system.insert_update(begin)
        .before(user)
        .before(graphics::acquire);
}

// Display system will simply display the Egui elements to the screen
pub fn display(system: &mut System) {
    system.insert_update(finish)
        .after(post_user)
        .before(graphics::present)
        .after(rendering::systems::composite::system);
}