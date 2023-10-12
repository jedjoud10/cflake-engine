use phobos::sync::submit_batch::SubmitBatch;
use winit::{event_loop::EventLoop, event::WindowEvent};
use world::{prelude::{Plugin, Init, Update}, world::World, system::{Registries, pre_user, post_user}, resource::State};
use crate::context::{WindowSettings, Window, Graphics};

/// Initialization function
pub fn init(world: &mut World, _: &Init) {
    let settings = world.remove::<WindowSettings>().unwrap();
    let el = world.get::<EventLoop<()>>().unwrap();
    let (window, graphics) = crate::context::initialize_phobos_context(&el, settings);
    drop(el);
    world.insert(window);
    world.insert(graphics);
}

/// Handle window quitting and resizing
pub fn window_event(world: &mut World, event: &WindowEvent) {
    match event {
        // Window has been resized
        WindowEvent::Resized(size) => {
            // Check if the size is valid
            if size.height == 0 || size.height == 0 {
                return;
            }
        }

        // Close requested, set the world state to "Stopped"
        WindowEvent::CloseRequested => {
            let mut state = world.get_mut::<State>().unwrap();
            *state = State::Stopped;
        }

        _ => (),
    }
}

/// Acquire system that will fetch the current swapchain image
pub fn acquire(world: &mut World, _: &Update) {
    let graphics = world.get::<Graphics>().unwrap();
    let mut _window = world.get_mut::<Window>().unwrap();
    let window = &mut *_window;
    let surface = &window.surface;
    let ifc = window.frame_manager.begin(&window.raw, graphics.exec.clone(), surface).unwrap();
    drop(_window);
    drop(graphics);
    world.insert(ifc);
}

/// Present system that will present the currently acquired swapchain image
pub fn present(world: &mut World, _: &Update) {
    use phobos::prelude::*;

    let ifc = world.remove::<phobos::InFlightContext>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut _window = world.get_mut::<crate::Window>().unwrap();
    let window = &mut *_window;
    let surface = &window.surface;
    
    let swapchain = phobos::image!("swapchain");

    let clear_pass = PassBuilder::<domain::All>::render("clear")
        .clear_color_attachment(&swapchain, ClearColor::Float([1.0; 4])).unwrap()
        .build(); 

    let present_pass = PassBuilder::present("present", clear_pass.output(&swapchain).unwrap());
    
    let mut graph = PassGraph::<domain::All>::new()
        .add_pass(clear_pass).unwrap()
        .add_pass(present_pass).unwrap()
        .build().unwrap();

    let mut bindings = PhysicalResourceBindings::new();
    bindings.bind_image("swapchain", &ifc.swapchain_image);
    let mut local_pool = phobos::pool::LocalPool::new(graphics.pool.clone()).unwrap();
    let cmd = graphics.exec.on_domain::<domain::All>().unwrap();
    let final_cmd = graph.record(
        cmd,
        &bindings,
        &mut local_pool,
        graphics.debug_messenger.clone(),
        &mut ()
    ).unwrap().finish().unwrap();

    let mut batch = graphics.exec.start_submit_batch::<domain::All>().unwrap();
    batch.submit_for_present(final_cmd, ifc, local_pool).unwrap();
    window.frame_manager.end(graphics.exec.clone(), batch).unwrap();
    drop(_window);
    drop(graphics);
}

/// Graphics plugin that will create the [phobos] context and [winit] window
pub fn plugin(registries: &mut Registries) {
    registries.init.insert(init).before(pre_user);
    registries.window_event.insert(window_event).before(pre_user);
    registries.update.insert(acquire).before(pre_user);
    registries.update.insert(present).after(post_user);
}

