use ::systems::*;
use assets::*;
use debug::*;
use defaults::{components, systems};
use ecs::*;
use errors::*;
use gl;
use glfw::{self, Context};
use input::*;
use io::SaverLoader;
use others::*;
use rendering::*;
use ui::UIManager;
use world_data::*;

use crate::GameConfig;
//  The actual world
pub struct World {
    // Managers
    pub component_manager: ComponentManager,
    pub input_manager: InputManager,
    pub ui_manager: UIManager,
    pub asset_manager: AssetManager,
    // ECS
    pub entity_manager: EntityManager,
    pub system_manager: SystemManager,

    // Miscs
    pub debug: MainDebug,
    pub custom_data: CustomWorldData,
    pub instance_manager: others::InstanceManager,
    pub time_manager: Time,
    pub saver_loader: SaverLoader,
    pub config_file: GameConfig,
}

impl World {
    // Get a new copy of a brand new world
    pub fn new(author_name: &str, app_name: &str) -> World {
        Self {
            component_manager: ComponentManager::default(),
            input_manager: InputManager::default(),
            ui_manager: UIManager::default(),
            asset_manager: AssetManager::default(),
            debug: MainDebug::default(),

            entity_manager: EntityManager::default(),
            system_manager: SystemManager::default(),

            instance_manager: InstanceManager::default(),
            custom_data: CustomWorldData::default(),
            time_manager: Time::default(),
            saver_loader: SaverLoader::new(author_name, app_name),
            config_file: GameConfig::default(),
        }
    }
    // Load everything that needs to be loaded by default
    fn load_defaults(&mut self, window: &mut glfw::Window) {
        // Load all the default things
        // Load default bindings
        self.input_manager.create_key_cache();
        self.input_manager.bind_key(Keys::F2, "debug_info", MapType::Button);
        self.input_manager.bind_key(Keys::F4, "toggle_console", MapType::Button);
        self.input_manager.bind_key(Keys::Enter, "enter", MapType::Button);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos(0.0, 0.0);

        // Load the default objects for the CacheManagers
        // Create the black texture
        Texture::new()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .enable_mipmaps()
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_name("black")
            .generate_texture(vec![0, 0, 0, 255])
            .unwrap()
            .object_cache_load("black", &mut self.asset_manager.object_cacher);
        // Create the white texture
        Texture::new()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_name("white")
            .generate_texture(vec![255, 255, 255, 255])
            .unwrap()
            .object_cache_load("white", &mut self.asset_manager.object_cacher);
        // Create the default normals texture
        Texture::new()
            .set_dimensions(TextureType::Texture2D(1, 1))
            .set_filter(TextureFilter::Linear)
            .enable_mipmaps()
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_name("default_normals")
            .generate_texture(vec![127, 128, 255, 255])
            .unwrap()
            .object_cache_load("default_normals", &mut self.asset_manager.object_cacher);        

        // Create some default UI that prints some default info to the screen
        let mut root = ui::Root::new(1);
        // ----Add the elements here----

        // Create a text element
        for x in 0..8 {
            let text_element_1 = ui::Element::new()
                .set_coordinate_system(ui::CoordinateType::Pixel)
                .set_position(veclib::Vector2::Y * 40.0 * x as f32)
                .set_text("", 40.0);
            root.add_element(text_element_1);
        }

        // Set this as the default root
        self.ui_manager.set_default_root(root);

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
        self.ui_manager.add_root("console", console_root);
    }
    // When the world started initializing
    pub fn start_world(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window, callback: fn(&mut Self)) {
        // A
        rendering::Utils::start_error_check_loop();

        // Load the default stuff
        self.load_defaults(window);

        // Load the config file for this world
        self.saver_loader.create_default("config\\game_config.che", &GameConfig::default());
        let mut config_file_values = self.saver_loader.load::<GameConfig>("config\\game_config.che");
        //config_file_values.vsync = true;
        self.config_file = config_file_values;

        // Enable disable vsync

        if self.config_file.vsync {
            // Enable VSync
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        } else {
            // Disable VSync
            glfw.set_swap_interval(glfw::SwapInterval::None);
        }
        //glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        // Set the window mode
        self.set_fullscreen(self.config_file.fullscreen, glfw, window);

        // Update entity manager
        self.update_entity_manager();

        self.custom_data.light_dir = veclib::Vector3::<f32>::new(0.0,1.0, 2.0).normalized();

        // Callback
        callback(self);
    }
    // We do the following in this function
    pub fn update_world(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw, delta: f64) {
        // Check for default input events
        self.check_default_input_events(window, glfw);
        // Upate the console
        self.update_console();
        // Create the data for the systems
        let mut data: WorldData = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            asset_manager: &mut self.asset_manager,
            input_manager: &mut self.input_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };

        // Update the system
        self.system_manager.update_systems(&mut data);
        window.swap_buffers();

        // Update the inputs
        self.input_manager.late_update(self.time_manager.delta_time as f32);

        // Update entity manager
        self.update_entity_manager();

        // Update the default UI
        let root = self.ui_manager.get_default_root_mut();
        let fps_text = &format!("FPS: {}", self.time_manager.average_fps.round());
        root.get_element_mut(1).update_text(fps_text, 40.0);
        let entity_text = &format!("#Entities: {}", self.entity_manager.entities.count_valid());
        root.get_element_mut(2).update_text(entity_text, 40.0);
        let entity_text = &format!("#Invalid Entities: {}", self.entity_manager.entities.count_invalid());
        root.get_element_mut(3).update_text(entity_text, 40.0);
        let x: &[Option<Entity>] = &self.entity_manager.entities.elements;
        let entity_text = &format!("#Valid Entities Byte Size: {}", std::mem::size_of_val(x));
        root.get_element_mut(4).update_text(entity_text, 40.0);
        let component_text = &format!("#Components: {}", self.component_manager.smart_components_list.count_valid());
        root.get_element_mut(5).update_text(component_text, 40.0);
        let component_text = &format!("#Invalid Components: {}", self.component_manager.smart_components_list.count_invalid());
        root.get_element_mut(6).update_text(component_text, 40.0);
        let component_text = &format!("#Valid Components Byte Size: {}", self.component_manager.smart_components_list.size_in_bytes);
        root.get_element_mut(7).update_text(component_text, 40.0);

        // Update the time
        self.time_manager.delta_time = delta;
        self.time_manager.seconds_since_game_start += delta;
        self.time_manager.frame_count += 1;
        // Update the FPS
        self.time_manager.fps = 1.0 / self.time_manager.delta_time;
        self.time_manager.update_average_fps();

        // Check for default mapping events
        if self.debug.console.listen_command("quit").is_some() {
            window.set_should_close(true);
        }
        // Toggle the fullscreen
        if self.debug.console.listen_command("toggle-fullscreen").is_some() {
            self.toggle_fullscreen(glfw, window);
        }
        // Toggle the rendering
        if self.debug.console.listen_command("toggle-render").is_some() {
            let rendering_system = self.system_manager.get_system_mut(0).unwrap();
            rendering_system.disable();
        }
    }
    // Check for default key map events
    fn check_default_input_events(&mut self, _window: &mut glfw::Window, _glfw: &mut glfw::Glfw) {
        // Debug world info (Component count, entity count, system count, fps, delta, and the rest)
        if self.input_manager.map_pressed("debug_info") {
            println!("Component count: '{}'", self.component_manager.smart_components_list.count_valid());
            println!("Entity count: '{}'", self.entity_manager.entities.count_valid());
            println!("System count: '{}'", self.system_manager.systems.len());
            println!(
                "Time: '{}', Delta Time: '{}', FPS: '{}', Frame Count: {}",
                self.time_manager.seconds_since_game_start, self.time_manager.delta_time, self.time_manager.fps, self.time_manager.frame_count
            );
        }
    }
    // Update the console
    fn update_console(&mut self) {
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
    }
    // Set the fullscreen status
    fn set_fullscreen(&mut self, fullscreen: bool, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        self.custom_data.window.fullscreen = fullscreen;
        if self.custom_data.window.fullscreen {
            // Set the glfw window as a fullscreen window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let videomode = monitor.unwrap().get_video_mode().unwrap();
                window.set_monitor(glfw::WindowMode::FullScreen(monitor.unwrap()), 0, 0, videomode.width, videomode.height, None);
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, videomode.width as i32, videomode.height as i32);
                }
            });
        } else {
            // Set the glfw window as a windowed window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let _videomode = monitor.unwrap().get_video_mode().unwrap();
                let default_window_size = others::get_default_window_size();
                window.set_monitor(glfw::WindowMode::Windowed, 50, 50, default_window_size.0 as u32, default_window_size.1 as u32, None);
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, default_window_size.0 as i32, default_window_size.1 as i32);
                }
            });
        }
    }
    // Toggle fullscreen
    fn toggle_fullscreen(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        self.custom_data.window.fullscreen = !self.custom_data.window.fullscreen;
        self.set_fullscreen(self.custom_data.window.fullscreen, glfw, window);

        // Enable disable vsync
        if self.config_file.vsync {
            // Enable VSync
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        } else {
            // Disable VSync
            glfw.set_swap_interval(glfw::SwapInterval::None);
        }
        println!("{}", self.config_file.vsync);
    }
    // When we want to close the application
    pub fn kill_world(&mut self) {
        let mut data: WorldData = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            asset_manager: &mut self.asset_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };
        self.system_manager.kill_systems(&mut data);
    }
}

// Impl block for the entity stuff
impl World {
    // Add the specified entity ID to the systems that it needs
    pub fn add_entity_to_systems(&mut self, entity_id: usize) {
        let entity = self.entity_manager.get_entity(entity_id).unwrap().clone();
        // Since we cloned the entity variable we gotta update the entity manager with the new one
        self.system_manager.add_entity_to_systems(
            &entity,
            &mut WorldData {
                entity_manager: &mut self.entity_manager,
                component_manager: &mut self.component_manager,
                ui_manager: &mut self.ui_manager,
                input_manager: &mut self.input_manager,
                asset_manager: &mut self.asset_manager,
                time_manager: &mut self.time_manager,
                debug: &mut self.debug,
                custom_data: &mut self.custom_data,
                instance_manager: &mut self.instance_manager,
            },
        );
        *self.entity_manager.get_entity_mut(entity_id).unwrap() = entity;
    }
    // Add all the pending entities from the entity manager to the systems and remove the ones that we must destroy
    pub fn update_entity_manager(&mut self) {
        // Only update if it we need to
        if self.entity_manager.entities_to_add.len() > 0 || self.entity_manager.entities_to_remove.len() > 0 {
            // Add the entities to the systems
            for entity in self.entity_manager.entities_to_add.clone() {
                self.add_entity_to_systems(entity.entity_id);
            }
            self.entity_manager.entities_to_add.clear();

            // Remove the entities from the systems
            for entity_id in self.entity_manager.entities_to_remove.clone() {
                self.remove_entity_from_systems(entity_id).unwrap();
                // After removing it from the systems, we can actually remove the entity
                self.entity_manager.entities.remove_element(entity_id);
            }
            self.entity_manager.entities_to_remove.clear();
        }
    }
    // Remove the specified entity ID from the systems it was in
    pub fn remove_entity_from_systems(&mut self, entity_id: usize) -> Result<Entity, ECSError> {
        // Remove this entity from the systems it was in first
        let entity = self.entity_manager.get_entity(entity_id)?.clone();
        let mut data = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            asset_manager: &mut self.asset_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };
        self.system_manager.remove_entity_from_systems(&entity, entity_id, &mut data);
        // Then remove the actual entity last, so it allows for systems to run their entity_removed event
        // Remove all the components then entity had
        for global_component_id in entity.linked_components.values() {
            self.component_manager.id_remove_linked_component(*global_component_id).unwrap();
        }
        Ok(entity)
    }
}

// Impl block related to the windowing / rendering stuff
impl World {
    // When we resize the window
    pub fn resize_window_event(&mut self, size: (u16, u16)) {
        self.custom_data.window.dimensions = veclib::Vector2::new(size.0, size.1);
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);

            let render_system = self
                .system_manager
                .get_custom_system_data_mut::<systems::rendering_system::CustomData>(self.custom_data.render_system_id)
                .unwrap();
            // Update the size of each texture that is bound to the framebuffer
            let dims = TextureType::Texture2D(size.0, size.1);
            render_system.diffuse_texture.update_size(dims);
            render_system.depth_texture.update_size(dims);
            render_system.normals_texture.update_size(dims);
            render_system.position_texture.update_size(dims);

            //TODO: This
            render_system.volumetric.update_texture_resolution(self.custom_data.window.dimensions);
        }
        let camera_entity_clone = self.entity_manager.get_entity(self.custom_data.main_camera_entity_id).unwrap().clone();
        let entity_clone_id = camera_entity_clone.entity_id;
        let camera_component = camera_entity_clone.get_component_mut::<components::Camera>(&mut self.component_manager).unwrap();
        camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
        camera_component.update_projection_matrix(&self.custom_data.window);
        camera_component.update_frustum_culling_matrix();
        // Update the original entity
        *self.entity_manager.get_entity_mut(entity_clone_id).unwrap() = camera_entity_clone;
    }
}
