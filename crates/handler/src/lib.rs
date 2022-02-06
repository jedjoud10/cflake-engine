#![windows_subsystem = "windows"]
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

extern crate gl;
// include the OpenGL type aliases
extern crate glfw;

// World
pub use defaults;
use glfw::WindowHint;
use main::core::World;
pub use main::*;

// Initialize GLFW and the Window
fn init_glfw(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
    // Set the type of events that we want to listen to
    use glfw::Context as GlfwContext;
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_scroll_polling(true);
    window.set_size_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::None);
}

// Load up the OpenGL window and such
pub fn start(author_name: &str, app_name: &str, preload_assets: fn(), init_world: fn(&mut World)) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(1, 0));
    let (mut window, events) = glfw
        .create_window(
            rendering::utils::DEFAULT_WINDOW_SIZE.x as u32,
            rendering::utils::DEFAULT_WINDOW_SIZE.y as u32,
            &format!("'{}', by '{}'", app_name, author_name),
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");
    init_glfw(&mut glfw, &mut window);
    // Pre-load the assets first
    defaults::preload_default_assets();
    preload_assets();
    
    // Load the config file (create it if it doesn't exist already)
    let io = main::io::SaverLoader::new(author_name, app_name);
    io.create_default("config\\game_config.json", &core::GameSettings::default());
    let config: core::GameSettings = io.load("config\\game_config.json");
    io.save("config\\game_config.json", &config);


    // Hehe multithreaded renering goes BRRRRRRRR
    let pipelin_settings = rendering::pipeline::PipelineSettings {
        shadow_resolution: config.shadow_resolution.convert(),
    };
    let pipeline_data = rendering::pipeline::init_pipeline(pipelin_settings, &mut glfw, &mut window);
    // Create the world
    let mut task_receiver = core::WorldTaskReceiver::new();
    let mut world = World::new(config, io, pipeline_data);

    // Init the world
    // Calling the callback
    println!("Calling World Initialization callback");
    {
        // Load the default systems first
        defaults::preload_system(&mut world);
        init_world(&mut world);
        // Flush everything and execute all the tasks
        task_receiver.flush(&mut world);
    }
    println!("Hello Game World!");
    while !window.should_close() {
        // Update the delta_time
        let new_time = glfw.get_time();
        // Update the timings
        world.time.update(new_time);
        // Get the GLFW events first
        poll_glfw_events(&mut glfw, &events, &mut world, &mut window);

        // We can update the world now
        World::update_start(&mut world, &mut task_receiver);
        World::update_end(&mut world, &mut task_receiver);
    }
    // When the window closes and we exit from the game
    println!("Exiting the engine...");
    world.destroy();
    println!("\x1b[31mThe sense of impending doom is upon us.\x1b[0m");
}
// Poll the glfw events first
fn poll_glfw_events(glfw: &mut glfw::Glfw, events: &std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>, world: &mut World, window: &mut glfw::Window) {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::Key(key, key_scancode, action_type, _modifiers) => {
                // Key event
                let action_id = match action_type {
                    glfw::Action::Press => 0,
                    glfw::Action::Release => 1,
                    glfw::Action::Repeat => 2,
                };
                // Only accept the scancode of valid keys
                if key_scancode > 0 {
                    world.input.receive_key_event(key_scancode, action_id);
                }
                if let glfw::Key::Escape = key {
                    window.set_should_close(true);
                }
            }
            glfw::WindowEvent::Size(x, y) => world.resize_window_event(veclib::Vector2::new(x as u16, y as u16)),
            glfw::WindowEvent::Scroll(_, scroll) => world.input.receive_mouse_scroll_event(scroll),
            glfw::WindowEvent::CursorPos(x, y) => world.input.receive_mouse_position_event((x, y)),
            _ => {}
        }
    }
}
