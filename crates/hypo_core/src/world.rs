use gl;
use glfw::{self, Context};
use hypo_debug::*;
use hypo_defaults::components;
use hypo_defaults::systems;
use hypo_ecs::*;
use hypo_errors::*;
use hypo_input::*;
use hypo_others::*;
use hypo_rendering::*;
use hypo_resources::*;
use hypo_system_event_data::*;
use hypo_systems::*;
use hypo_ui::UIManager;
use std::collections::HashSet;
//  The actual world
#[derive(Default)]
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
    pub time_manager: Time,
}

impl World {
    // Load everything that needs to be loaded by default
    fn load_defaults(&mut self, window: &mut glfw::Window) {
        // Load all the default things
        // Load default bindings
        self.input_manager.bind_key(Keys::Escape, "quit", MapType::Button);
        self.input_manager.bind_key(Keys::F1, "fullscreen", MapType::Button);
        self.input_manager.bind_key(Keys::F2, "debug_info", MapType::Button);
        self.input_manager.bind_key(Keys::F3, "change_debug_view", MapType::Button);
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
                vec!["defaults\\shaders\\default.vrsh.glsl", "defaults\\shaders\\default.frsh.glsl"],
                &mut self.resource_manager,
                &mut self.shader_cacher,
            );
            default_shader_name = default_shader.1;
        }
        self.shader_cacher.1.generate_defaults(vec![default_shader_name.as_str()]);
    }
    // When the world started initializing
    pub fn start_world(&mut self, window: &mut glfw::Window, load_systems_callback: fn(&mut Self), load_entities_callback: fn(&mut Self)) {
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
        load_systems_callback(self);
        // Update entity manager
        self.update_entity_manager();
        load_entities_callback(self);
    }
    // We do the following in this function
    // 1. We update the entities of each UpdateSystem
    // 2. We tick the entities of each TickSystem (Only if the framecount is valid)
    // 3. We render the entities onto the screen using the RenderSystem
    pub fn update_world(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
        // Check for input events
        self.input_manager.update();
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
                "Time: '{}', Delta Time: '{}', FPS: '{}'",
                self.time_manager.seconds_since_game_start, self.time_manager.delta_time, self.time_manager.fps
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

        // Update the FPS
        self.time_manager.fps = 1.0 / self.time_manager.delta_time;
    }
    // Toggle fullscreen
    pub fn toggle_fullscreen(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        self.custom_data.window.fullscreen = !self.custom_data.window.fullscreen;
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
                let default_window_size = hypo_others::get_default_window_size();
                window.set_monitor(glfw::WindowMode::Windowed, 50, 50, default_window_size.0 as u32, default_window_size.1 as u32, None);
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, default_window_size.0 as i32, default_window_size.1 as i32);
                }
            });
        }
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
