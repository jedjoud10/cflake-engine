use crate::{custom_world_data::CustomWorldData, GameConfig};
use ::rendering::*;

use debug::*;
use ecs::*;
use glfw::{self};
use input::*;
use io::SaverLoader;
use others::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::ThreadId;
use ui::UIManager;

// Global main for purely just low level task management
use lazy_static::lazy_static;
lazy_static! {
    static ref WORLD: RwLock<World> = RwLock::new(new_internal());
    static ref FRAME: AtomicBool = AtomicBool::new(false);
}

// Check if we are currently running a frame
pub fn check_frame() -> bool {
    FRAME.load(Ordering::Relaxed)
}

// Get a reference to the world
pub fn world() -> RwLockReadGuard<'static, World> {
    let x = WORLD.read().unwrap();
    x
}

// Get a mutable reference to the world
pub fn world_mut() -> RwLockWriteGuard<'static, World> {
    // Check if we are in the middle of a frame
    if FRAME.load(std::sync::atomic::Ordering::Relaxed) {
        // We are currently running a frame, we cannot get the world mutably
        panic!("Cannot get the world mutably during a frame!");
    }
    let x = WORLD.write().unwrap();
    x
}

//  The actual world
pub struct World {
    pub input_manager: InputManager,
    pub ui_manager: UIManager,
    pub ecs_manager: ECSManager,

    // Miscs
    pub debug: MainDebug,
    pub instance_manager: others::InstanceManager,
    pub custom_data: CustomWorldData,
    pub time_manager: Time,
    pub saver_loader: SaverLoader,
    pub config_file: GameConfig,
}

// Get a new copy of a brand new world (Though don't initialize the SaverLoader yet)
pub fn new_internal() -> World {
    World {
        ecs_manager: ECSManager::default(),
        input_manager: InputManager::default(),
        ui_manager: UIManager::default(),
        debug: MainDebug::default(),

        instance_manager: InstanceManager::default(),
        custom_data: CustomWorldData::default(),
        time_manager: Time::default(),
        saver_loader: SaverLoader::default(),
        config_file: GameConfig::default(),
    }
}
// Just update the saver loader basically
pub fn new(author_name: &str, app_name: &str) {
    println!("Going to create a new SaverLoader");
    let mut w = world_mut();
    w.saver_loader = SaverLoader::new(author_name, app_name);
}
// Just create a new saver loader
// When the world started initializing
pub fn start_world(glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
    println!("Starting world...");
    // Start the multithreaded shit
    crate::command::initialize_channels_main();
    // Load the default stuff
    crate::local::input::create_key_cache();
    crate::global::input::bind_key(Keys::F4, "toggle_console", MapType::Button);
    crate::global::input::bind_key(Keys::Enter, "enter", MapType::Button);

    // Load the default objects for the CacheManagers
    // Load the missing texture
    pipec::texturec(assets::cachec::acache_l("defaults\\textures\\missing_texture.png", Texture::default().enable_mipmaps()).unwrap());
    // Create the black texture
    pipec::texturec(
        assets::cachec::cache(
            "black",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("black")
                .set_bytes(vec![0, 0, 0, 255]),
        )
        .unwrap(),
    );

    // Create the white texture
    pipec::texturec(
        assets::cachec::cache(
            "white",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("white")
                .set_bytes(vec![255, 255, 255, 255]),
        )
        .unwrap(),
    );
    // Create the default normals texture
    pipec::texturec(
        assets::cachec::cache(
            "default_normals",
            Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("default_normals")
                .set_bytes(vec![127, 128, 255, 255]),
        )
        .unwrap(),
    );

    // Create some default UI that prints some default info to the screen
    let mut root = ui::Root::new(1);
    // ----Add the elements here----

    // Create a text element
    for x in 0..2 {
        let text_element_1 = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::Y * 40.0 * x as f32)
            .set_text("", 40.0);
        root.add_element(text_element_1);
    }

    // Set this as the default root
    crate::global::ui::add_root("default", root);

    // Create the default root for the console
    let mut console_root = ui::Root::new(64);
    let console_panel = ui::Element::new()
        .set_coordinate_system(ui::CoordinateType::Factor)
        .set_color(veclib::Vector4::new(0.0, 0.0, 0.0, 0.7));
    let console_panel_id = console_root.add_element(console_panel);
    let console_text = ui::Element::new()
        .set_coordinate_system(ui::CoordinateType::Pixel)
        .set_position(veclib::Vector2::ZERO)
        .set_size(veclib::Vector2::ONE)
        .set_text("text", 30.0);
    let console_text_id = console_root.add_element(console_text);
    ui::Element::attach(&mut console_root, console_panel_id, vec![console_text_id]);
    console_root.visible = false;
    crate::global::ui::add_root("console", console_root);
    window_commands::hide_cursor(window);
    // Load the config file for this world
    let config_file_copy = crate::global::io::create_config_file();
    // Apply the config file's data to the rendering window
    window_commands::set_fullscreen(config_file_copy.fullscreen, glfw, window);
    window_commands::set_vsync(config_file_copy.vsync);
    println!("Hello world from MainThread! Must call initalization callback!");
}
// This is the main Update loop, ran on the main thread
pub fn update_world_start_barrier(delta: f64) {
    // Systems are halting, tell them to continue their next frame
    println!("Update world in {:.2}ms", delta * 1000.0);
    others::barrier::as_ref().thread_sync();
    FRAME.store(true, Ordering::Relaxed);
    // The systems are running, we cannot do anything main thread related
}
// Finish the frame, telling the logic systems to wait until they all sync up
pub fn update_world_end_barrier(delta: f64, thread_ids: &Vec<ThreadId>) {
    FRAME.store(false, Ordering::Relaxed);
    // --- SYSTEM FRAME END HERE ---
    // We will tell the systems to execute their local callbacks
    for thread_id in thread_ids {
        others::barrier::as_ref().thread_sync_local_callbacks(thread_id);
        // Wait until the special block finish
        others::barrier::as_ref().thread_sync_local_callbacks(thread_id);
        // --- THE SYSTEM FRAME LOOP ENDS, IT GOES BACK TO THE TOP OF THE LOOP ---
    }
    // The sytems started halting, we can do stuff on the main thread
}
// Update main thread stuff
pub fn update_main_thread_stuff(delta: f64, world: &mut World, pipeline_start_data: &PipelineStartData) {
    // Run the commands at the end of the frame
    crate::command::frame_main_thread(world, pipeline_start_data);
    world.input_manager.late_update(delta as f32);
    world.time_manager.elapsed = world.time_manager.elapsed + delta;
    world.time_manager.delta_time = delta;
}

// Update the console
fn update_console() {
    /*
    // Check if we should start key registering if the console is active
    if self.input_manager.map_pressed_uncheck("toggle_console") || (self.input_manager.map_pressed_uncheck("enter") && self.input_manager.keys_reg_active()) {
        match self.input_manager.toggle_keys_reg() {
            Some(x) => {
                // Hide the console
                let console_root = self.ui_manager.get_root_mut("console");
                console_root.visible = false;
                self.debug.console.detect_command(x);
            }
            None => {
                // Enable the console
                let console_root = self.ui_manager.get_root_mut("console");
                console_root.visible = true;
            }
        }
    }

    // Update the console everytime
    match self.input_manager.full_sentence.as_ref() {
        Some(x) => {
            let console_text = self.ui_manager.get_root_mut("console").get_element_mut(2);
            let console_string = format!("Com: '{}'", x.clone().as_str());
            console_text.update_text(console_string.as_str(), 40.0);
        }
        None => {
            // We don't have to update anything
        }
    }
    */
}
// When we want to close the application
pub fn kill_world(pipeline_data: PipelineStartData) {
    println!("Killing child threads...");
    let barrier_data = others::barrier::as_ref();

    // Run their last frame...
    println!("Loop threads running their last frame...");
    // Set the AtomicBool
    barrier_data.destroying_world();
    FRAME.store(false, Ordering::Relaxed);
    barrier_data.thread_sync();
    barrier_data.thread_sync_quit();
    println!("Loop threads ran their last frame!");

    let mut w = world_mut();
    let systems = std::mem::take(&mut w.ecs_manager.systemm.systems);
    // Tell all the child loop threads to stop
    // Then we join them
    for data in systems {
        data.join_handle.join().unwrap();
    }
    pipec::join_pipeline(pipeline_data);
    println!("Joined up all the child threads, we can safely exit!");
}
// We have received input events from GLFW
pub fn receive_key_event(key_scancode: i32, action_type: i32, world: &mut World) {
    world.input_manager.receive_key_event(key_scancode, action_type);
}
pub fn receive_mouse_pos_event(x: f64, y: f64, world: &mut World) {
    world.input_manager.receive_mouse_event(Some((x, y)), None);
}
pub fn receive_mouse_scroll_event(scroll: f64, world: &mut World) {
    world.input_manager.receive_mouse_event(None, Some(scroll));
}
pub fn resize_window_event(_x: u16, _y: u16, _world: &mut World) {}
/*
// When we resize the window
pub fn resize_window_event(size: (u16, u16)) {
    let dims = veclib::Vector2::new(size.0, size.1);
    pipec::task(pipec::RenderTask::WindowUpdateSize(dims), "window_data_update", |_| {});
    let world = crate::world::world_mut();
    let camera_entity_clone = crate::global::ecs::entity(world.custom_data.main_camera_entity_id).unwrap();
    let entity_clone_id = camera_entity_clone.entity_id;
    let camera_component = crate::global::ecs::component_mut::<components::Camera>(camera_entity_clone, |x| {

    }).unwrap();
    camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
    camera_component.update_aspect_ratio(dims);
}
*/
