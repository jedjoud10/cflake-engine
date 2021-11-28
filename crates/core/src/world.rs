use ::rendering::*;
use ::systems::*;
use assets::*;
use debug::*;
use defaults::{components, systems};
use ecs::*;
use glfw::{self, Context};
use input::*;
use io::SaverLoader;
use others::*;
use ui::UIManager;
use world_data::*;

use crate::GameConfig;
//  The actual world
pub struct World {
    // Managers
    pub component_manager: ComponentManager,
    pub input_manager: InputManager,
    pub ui_manager: UIManager,
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
    fn load_defaults(&mut self) {
        // Load default bindings
        self.input_manager.create_key_cache();
        self.input_manager.bind_key(Keys::F4, "toggle_console", MapType::Button);
        self.input_manager.bind_key(Keys::Enter, "enter", MapType::Button);

        // Load the default objects for the CacheManagers
        // Load the missing texture 
        pipec::texturec(
            assets::cachec::acache_l("defaults\\textures\\missing_texture.png", Texture::default().enable_mipmaps()).unwrap()
        );
        // Create the black texture
        pipec::texturec(
            assets::cachec::cache("black", Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("black")
                .set_bytes(vec![0, 0, 0, 255])).unwrap()
        );
        // Create the white texture
        pipec::texturec(
            assets::cachec::cache("white", Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("white")
                .set_bytes(vec![255, 255, 255, 255])).unwrap()
        );
        // Create the default normals texture
        pipec::texturec(
            assets::cachec::cache("default_normals", Texture::default()
                .set_dimensions(TextureType::Texture2D(1, 1))
                .set_filter(TextureFilter::Linear)
                .enable_mipmaps()
                .set_name("default_normals")
                .set_bytes(vec![127, 128, 255, 255])).unwrap(),
        );

        // Load the default systems
        let mut data: WorldData = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };
        // Rendering system
        let mut rendering_system = systems::rendering_system::system(&mut data);
        rendering_system.enable(&mut data);
        self.system_manager.add_system(rendering_system);
        // Camera system
        let mut camera_system = systems::camera_system::system(&mut data);
        camera_system.enable(&mut data);
        self.system_manager.add_system(camera_system);
        /*
        let mut ui_system = systems::ui_system::system(&mut data);
        ui_system.enable(&mut data);
        self.system_manager.add_system(ui_system);
        */
        // Load the default command system
        let mut command_system = systems::command_system::system(&mut data);
        command_system.enable(&mut data);
        self.system_manager.add_system(command_system);
        // Load the terrain system
        /*
        let mut terrain_system = systems::terrain_system::system(&mut data);
        terrain_system.enable(&mut data);
        self.system_manager.add_system(terrain_system);
        */

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
    pub fn start_world(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window,  callback: fn(&mut Self)) {
        // Load the default stuff
        self.load_defaults();        
        window_commands::hide_cursor(window);
        // Load the config file for this world
        self.saver_loader.create_default("config\\game_config.json", &GameConfig::default());
        let config_file_values = self.saver_loader.load::<GameConfig>("config\\game_config.json");
        self.config_file = config_file_values;
        // Apply the config file's data to the rendering window
        window_commands::set_fullscreen(self.config_file.fullscreen, glfw, window);
        window_commands::set_vsync(self.config_file.vsync);
        // Update entity manager
        self.update_entity_manager();
        
        self.custom_data.light_dir = veclib::Vector3::<f32>::new(0.0, 1.0, 2.0).normalized();
        
        // Callback
        callback(self);
        println!("Hello world!");
    }
    // We do the following in this function
    pub fn update_world(&mut self, delta: f64, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        // Upate the console
        self.update_console();
        // Create the data for the systems
        let mut data: WorldData = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };

        // Update the system
        self.system_manager.update_systems(&mut data);

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

        // Update the time
        self.time_manager.delta_time = delta;
        self.time_manager.seconds_since_game_start += delta;
        self.time_manager.frame_count += 1;
        // Update the FPS
        self.time_manager.fps = 1.0 / self.time_manager.delta_time;
        self.time_manager.update_average_fps();

        // Check for default mapping events
        if self.debug.console.listen_command("quit").is_some() {
            self.kill_world();    
        }
        // Toggle the fullscreen
        if self.debug.console.listen_command("toggle-fullscreen").is_some() {
            self.custom_data.fullscreen = !self.custom_data.fullscreen;
            window_commands::set_fullscreen(self.custom_data.fullscreen, glfw, window);
        }
        // Toggle the rendering
        if self.debug.console.listen_command("toggle-render").is_some() {
            let rendering_system = self.system_manager.get_system_mut(0).unwrap();
            rendering_system.disable();
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
    // When we want to close the application
    pub fn kill_world(&mut self) {
        let mut data: WorldData = WorldData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            ui_manager: &mut self.ui_manager,
            input_manager: &mut self.input_manager,
            time_manager: &mut self.time_manager,
            debug: &mut self.debug,
            custom_data: &mut self.custom_data,
            instance_manager: &mut self.instance_manager,
        };
        self.system_manager.kill_systems(&mut data);
        println!("Kill world!");
        // Kill the render pipeline
        pipec::dispose_pipeline();
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
        let dims = veclib::Vector2::new(size.0, size.1);
        pipec::task(pipec::RenderTask::WindowUpdateSize(dims), "window_data_update", |_| { });
        let camera_entity_clone = self.entity_manager.get_entity(self.custom_data.main_camera_entity_id).unwrap().clone();
        let entity_clone_id = camera_entity_clone.entity_id;
        let camera_component = camera_entity_clone.get_component_mut::<components::Camera>(&mut self.component_manager).unwrap();
        camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
        camera_component.update_aspect_ratio(dims);
        // Update the original entity
        *self.entity_manager.get_entity_mut(entity_clone_id).unwrap() = camera_entity_clone;
    }
}

// Pre-load the default assets
pub fn preload_default_assets() {
    // Pre load the resources
    println!("Pre-loading default assets...");
    // Rendering
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\passthrough.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\screen.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\rendering\\default.frsh.glsl");
    // Others
    preload_asset!(".\\resources\\defaults\\shaders\\others\\wireframe.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\hashes.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\triplanar.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\debug.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\debug.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\frame_stats.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\others\\sdf.func.glsl");
    // UI
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_elem.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_panel.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.vrsh.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\ui\\ui_font.frsh.glsl");
    preload_asset!(".\\resources\\defaults\\fonts\\default_font.font");
    // Models
    preload_asset!(".\\resources\\defaults\\models\\screen_quad.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\sphere.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\quad.mdl3d");
    preload_asset!(".\\resources\\defaults\\models\\cube.mdl3d");
    // Noise
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\simplex.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\noises\\voronoi.func.glsl");
    // Voxel terrain
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\voxel_main.cmpt.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\data.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain_shader.func.glsl");
    preload_asset!(".\\resources\\defaults\\shaders\\voxel_terrain\\terrain.frsh.glsl");
    // Textures
    preload_asset!(".\\resources\\defaults\\textures\\missing_texture.png");
    preload_asset!(".\\resources\\defaults\\textures\\sky_gradient.png");
    preload_asset!(".\\resources\\defaults\\textures\\rock_diffuse.png");
    preload_asset!(".\\resources\\defaults\\textures\\rock_normal.png");
    println!("Finished pre-loading default assets!");    
}

