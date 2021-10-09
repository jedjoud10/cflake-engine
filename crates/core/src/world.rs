use ::systems::*;
use debug::*;
use defaults::components;
use defaults::systems;
use ecs::*;
use errors::*;
use fonts::FontManager;
use gl;
use glfw::{self, Context};
use input::*;
use io::SaverLoader;
use others::*;
use rendering::*;
use resources::*;
use std::collections::HashSet;
use system_event_data::*;
use ui::UIManager;

use crate::GameConfig;
//  The actual world
pub struct World {
    // Managers
    pub component_manager: ComponentManager,
    pub input_manager: InputManager,
    pub resource_manager: ResourceManager,
    pub ui_manager: UIManager,
    // Rendering
    pub texture_cacher: CacheManager<Texture2D>,
    pub shader_cacher: (CacheManager<SubShader>, CacheManager<Shader>),
    pub debug: DebugRenderer,
    // ECS
    pub entity_manager: EntityManager,
    pub system_manager: SystemManager,

    // Miscs
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
            resource_manager: ResourceManager::default(),
            ui_manager: UIManager::default(),

            texture_cacher: CacheManager::default(),
            shader_cacher: (CacheManager::default(), CacheManager::default()),
            debug: DebugRenderer::default(),

            entity_manager: EntityManager::default(),
            system_manager: SystemManager::default(),

            instance_manager: InstanceManager::default(),
            custom_data: CustomWorldData::default(),
            time_manager: Time::default(),
            saver_loader: SaverLoader::new(author_name, app_name),
            config_file: GameConfig::default()
        }
    }
    // Load everything that needs to be loaded by default
    fn load_defaults(&mut self, window: &mut glfw::Window) {
        // Load all the default things
        // Load default bindings
        self.input_manager.create_key_cache();
        self.input_manager.bind_key(Keys::Escape, "quit", MapType::Button);
        self.input_manager.bind_key(Keys::F1, "fullscreen", MapType::Button);
        self.input_manager.bind_key(Keys::F2, "debug_info", MapType::Button);
        self.input_manager.bind_key(Keys::F3, "change_debug_view", MapType::Button);
        self.input_manager.bind_key(Keys::F4, "toggle_console", MapType::Button);
        self.input_manager.bind_key(Keys::F, "toggle_wireframe", MapType::Button);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos(0.0, 0.0);

        // Load the default objects for the CacheManagers
        let _white_texture = Texture2D::new()
            .load_texture("defaults\\textures\\white.png", &mut self.resource_manager, &mut self.texture_cacher)
            .unwrap();
        let _black_texture = Texture2D::new()
            .load_texture("defaults\\textures\\black.png", &mut self.resource_manager, &mut self.texture_cacher)
            .unwrap();
        self.texture_cacher
            .generate_defaults(vec!["defaults\\textures\\white.png", "defaults\\textures\\black.png"]);

        // Copy the default shader name
        let default_shader_name: String;
        {
            let default_shader = Shader::new(
                vec!["defaults\\shaders\\rendering\\default.vrsh.glsl", "defaults\\shaders\\rendering\\default.frsh.glsl"],
                &mut self.resource_manager,
                &mut self.shader_cacher,
                None,
            );
            default_shader_name = default_shader.1;
        }
        self.shader_cacher.1.generate_defaults(vec![default_shader_name.as_str()]);

        // Create some default UI that prints some default info to the screen
        let mut root = ui::Root::new(1);
        // ----Add the elements here----

        // Create a text element
        let text_element_1 = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::ZERO)
            .set_text("fps_text_here", 40.0);
        root.add_element(text_element_1);
        let text_element_2 = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::Y * 40.0)
            .set_text("entity_text_here", 40.0);
        root.add_element(text_element_2);
        let text_element_3 = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Pixel)
            .set_position(veclib::Vector2::Y * 40.0 * 2.0)
            .set_text("component_text_here", 40.0);
        root.add_element(text_element_3);

        // Set this as the default root
        self.ui_manager.set_default_root(root);

        // Create the default root for the console
        let mut console_root = ui::Root::new(64);
        let console_panel = ui::Element::new()
            .set_coordinate_system(ui::CoordinateType::Factor)
            .set_color(veclib::Vector4::new(0.0, 0.0, 0.0, 1.0));
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
        // Load the default stuff
        self.load_defaults(window);
        /*
        // Test stuff
        self.component_manager.register_component::<components::Transform>();
        let mut test_entity = Entity::new("Test Entity");
        test_entity.link_default_component::<components::Transform>(&mut self.component_manager).unwrap();
        let entity_id = self.add_entity(test_entity);
        self.entity_manager.remove_entity_s(&entity_id);
        */        
        // Load the config file for this world
        self.saver_loader.create_default("config\\game_config.che", &GameConfig::default());
        let config_file_values = self.saver_loader.load::<GameConfig>("config\\game_config.che");
        self.config_file = config_file_values;

        // Enable disable vsync
        if self.config_file.vsync {
            // Enable VSync
            glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        } else {
            // Disable VSync
            glfw.set_swap_interval(glfw::SwapInterval::None);
        }

        // Set the window mode
        self.set_fullscreen(self.config_file.fullscreen, glfw, window);

        // Update entity manager
        self.update_entity_manager();

        // Callback
        callback(self);
    }
    // We do the following in this function
    // 1. We update the entities of each UpdateSystem
    // 2. We tick the entities of each TickSystem (Only if the framecount is valid)
    // 3. We render the entities onto the screen using the RenderSystem
    pub fn update_world(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw, delta: f64) {
        // Check for default input events
        self.check_default_input_events(window, glfw);
        // Create the data for the systems
        let mut data: SystemEventData = SystemEventData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            shader_cacher: &mut self.shader_cacher,
            texture_cacher: &mut self.texture_cacher,
            resource_manager: &mut self.resource_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager
        };

        // Update the entities
        self.system_manager.run_system_type(SystemType::Update, &mut data);
        // And render them
        self.system_manager.run_system_type(SystemType::Render, &mut data);
        window.swap_buffers();

        // Update the timings of every system
        self.system_manager.update_systems(&self.time_manager);

        // Update the inputs
        self.input_manager.late_update(self.time_manager.delta_time as f32);

        // Update entity manager
        self.update_entity_manager();

        // Update the default UI
        let root = self.ui_manager.get_default_root_mut();
        let fps_text = &format!("FPS: {}", self.time_manager.average_fps.round());
        root.get_element_mut(1).update_text(fps_text, 40.0);
        let entity_text = &format!("#Entities: {}", self.entity_manager.entities.len());
        root.get_element_mut(2).update_text(entity_text, 40.0);
        let component_text = &format!("#Components: {}", self.component_manager.smart_components_list.elements.len());
        root.get_element_mut(3).update_text(component_text, 40.0);        

        // Just in case
        errors::ErrorCatcher::catch_opengl_errors();

        // Update the time
        self.time_manager.delta_time = delta;
        self.time_manager.seconds_since_game_start += delta;
        self.time_manager.frame_count += 1;
        // Update the FPS
        self.time_manager.fps = 1.0 / self.time_manager.delta_time;
        self.time_manager.update_average_fps();
    }
    // Check for default key map events
    fn check_default_input_events(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
        // Check for default mapping events
        if self.input_manager.map_pressed("quit") {
            window.set_should_close(true);
        }
        // Toggle the fullscreen
        if self.input_manager.map_pressed("fullscreen") {
            self.toggle_fullscreen(glfw, window);
        }
        // Debug world info (Component count, entity count, system count, fps, delta, and the rest)
        if self.input_manager.map_pressed("debug_info") {
            println!("Component count: '{}'", self.component_manager.smart_components_list.elements.len());
            println!("Entity count: '{}'", self.entity_manager.entities.len());
            println!("System count: '{}'", self.system_manager.systems.len());
            println!(
                "Time: '{}', Delta Time: '{}', FPS: '{}', Frame Count: {}",
                self.time_manager.seconds_since_game_start, self.time_manager.delta_time, self.time_manager.fps, self.time_manager.frame_count
            );
        }
        // Change the debug view
        if self.input_manager.map_pressed("change_debug_view") {
            let render_system = self.system_manager.get_system_mut::<systems::RenderingSystem>(self.custom_data.render_system_id).unwrap();
            render_system.debug_view += 1;
            render_system.debug_view %= 4;
        }
        // Enable / Disable wireframe
        if self.input_manager.map_pressed("toggle_wireframe") {
            let render_system = self.system_manager.get_system_mut::<systems::RenderingSystem>(self.custom_data.render_system_id).unwrap();
            render_system.wireframe = !render_system.wireframe;
        }
        // Check if we should start key registering if the console is active
        if self.input_manager.map_pressed_uncheck("toggle_console") {
            match self.input_manager.toggle_keys_reg() {
                Some(x) => {
                    // Hide the console
                    let console_root = self.ui_manager.get_root_mut("console");
                    console_root.visible = false;          
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
                console_text.update_text(console_string.as_str(), 20.0);
            },
            None => {
                // We don't have to update anything
            },
        }
    }
    // Set the fullscreen status
    pub fn set_fullscreen(&mut self, fullscreen: bool, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
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
    pub fn toggle_fullscreen(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
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
        let mut data: SystemEventDataLite = SystemEventDataLite {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            custom_data: &mut self.custom_data,
        };
        self.system_manager.kill_systems(&mut data);
    }
}

// Impl block for the entity stuff
impl World {
    // Add the specified entity ID to the systems that it needs
    pub fn add_entity_to_systems(&mut self, entity_id: &u16) {
        let entity = self.entity_manager.get_entity(entity_id).unwrap().clone();
        // Since we cloned the entity variable we gotta update the entity manager with the new one
        self.system_manager.add_entity_to_systems(
            &entity,
            &mut SystemEventDataLite {
                entity_manager: &mut self.entity_manager,
                component_manager: &mut self.component_manager,
                custom_data: &mut self.custom_data,
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
                self.add_entity_to_systems(&entity.entity_id);
            }
            self.entity_manager.entities_to_add.clear();

            // Remove the entities from the systems
            for entity_id in self.entity_manager.entities_to_remove.clone() {
                self.remove_entity_from_systems(&entity_id).unwrap();
                // After removing it from the systems, we can actually remove the entity
                self.entity_manager.entities[entity_id as usize] = None;
            }
            self.entity_manager.entities_to_remove.clear();
        }
    }
    // Remove the specified entity ID from the systems it was in
    pub fn remove_entity_from_systems(&mut self, entity_id: &u16) -> Result<Entity, ECSError> {
        // Remove this entity from the systems it was in first
        let entity = self.entity_manager.get_entity(entity_id)?.clone();
        let mut data = SystemEventDataLite {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            custom_data: &mut self.custom_data,
        };
        self.system_manager.remove_entity_from_systems(&entity, entity_id, &mut data);
        // Then remove the actual entity last, so it allows for systems to run their entity_removed event
        // Remove all the components then entity had
        for global_component_id in entity.linked_components.values() {
            self.component_manager.id_remove_linked_component(global_component_id).unwrap();
        }
        Ok(entity)
    }
}

// Impl block related to the windowing / rendering stuff
impl World {
    // When we resize the window
    pub fn resize_window_event(&mut self, size: (u16, u16)) {
        self.custom_data.window.size = veclib::Vector2::new(size.0, size.1);
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);

            let render_system = self.system_manager.get_system_mut::<systems::RenderingSystem>(self.custom_data.render_system_id).unwrap();
            // Update the size of each texture that is bound to the framebuffer
            render_system.window.size = veclib::Vector2::new(size.0, size.1);
            render_system.diffuse_texture.update_size(size.0, size.1);
            render_system.depth_stencil_texture.update_size(size.0, size.1);
            render_system.normals_texture.update_size(size.0, size.1);
            render_system.position_texture.update_size(size.0, size.1);
            render_system.emissive_texture.update_size(size.0, size.1);
        }
        let camera_entity_clone = self.entity_manager.get_entity(&self.custom_data.main_camera_entity_id).unwrap().clone();
        let entity_clone_id = camera_entity_clone.entity_id;
        let camera_component = camera_entity_clone.get_component_mut::<components::Camera>(&mut self.component_manager).unwrap();
        camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
        camera_component.update_projection_matrix(&self.custom_data.window);
        camera_component.update_frustum_culling_matrix();
        // Update the original entity
        *self.entity_manager.get_entity_mut(&entity_clone_id).unwrap() = camera_entity_clone;
    }
}
