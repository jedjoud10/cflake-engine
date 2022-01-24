//#![windows_subsystem = "windows"]
extern crate gl;
// include the OpenGL type aliases
extern crate glfw;

// World
pub use defaults;
use main::core::{Context, TaskSenderContext, World, WriteContext};
pub use main::*;
use std::sync::{Arc, RwLock};

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
pub fn start(author_name: &str, app_name: &str, preload_assets: fn(), init_world: fn(WriteContext<'_>)) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
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
    // Hehe multithreaded renering goes BRRRRRRRR
    let pipeline_data = rendering::pipeline::init_pipeline(&mut glfw, &mut window);
    rendering::pipeline::init_coms();
    // Create the world
    let mut task_receiver = core::WorldTaskReceiver::new();
    let world = Arc::new(RwLock::new(World::new(author_name, app_name, pipeline_data)));

    // Init the world
    // Calling the callback
    println!("Calling World Initialization callback");
    {
        {
            let mut context = Context::convert(&world);
            // Load the default systems first
            defaults::preload_system(context.write());
            init_world(context.write());
            // Flush everything and execute all the tasks
        }
        {
            let mut world = world.write().unwrap();
            task_receiver.flush(&mut world);
        }
    }
    println!("Hello Game World!");
    while !window.should_close() {
        {
            // Update the delta_time
            let new_time = glfw.get_time();
            // Update the de
            let mut world = world.write().unwrap();
            world.time.update(new_time);
        }
        // Get the GLFW events first
        glfw.poll_events();
        {
            let mut world = world.write().unwrap();
            for (_, event) in glfw::flush_messages(&events) {
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
                    glfw::WindowEvent::Size(x, y) => {
                        world.resize_window_event(veclib::Vector2::new(x as u16, y as u16))
                    },
                    glfw::WindowEvent::Scroll(_, scroll) => world.input.receive_mouse_scroll_event(scroll),
                    glfw::WindowEvent::CursorPos(x, y) => world.input.receive_mouse_position_event((x, y)),
                    _ => {}
                }
            }
        }
        // We can update the world now
        World::update_start(&world, &mut task_receiver);
        World::update_end(&world, &mut task_receiver);
    }
    // When the window closes and we exit from the game
    if let Ok(rwlock) = Arc::try_unwrap(world) {
        println!("Exiting the engine...");
        let world = rwlock.into_inner().unwrap();
        world.destroy();
    } else {
        panic!("Nah bro you mad goofy");
    }
    println!("\x1b[31mThe sense of impending doom is upon us.\x1b[0m");
}
