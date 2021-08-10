use glfw::Context;
use std::collections::HashMap;
use std::ptr::null;

use crate::engine::core::ecs::component::*;
use crate::engine::core::ecs::entity::*;
use crate::engine::core::ecs::system::*;
use crate::engine::core::ecs::system_data::*;
use crate::engine::input::*;
use crate::engine::rendering::*;

use crate::engine::core::defaults::components::components::Camera;
use crate::engine::resources::ResourceManager;
use crate::game::level::*;

// Import stuff from the rendering module
use crate::engine::rendering::shader::ShaderManager;
use crate::engine::rendering::texture::TextureManager;

use super::defaults::components::transforms::Position;

//  The actual world
pub struct World {
    pub time_manager: Time,
    pub component_manager: ComponentManager,
    pub input_manager: InputManager,
    pub resource_manager: ResourceManager,
    pub shader_manager: ShaderManager,
    pub texture_manager: TextureManager,
    pub entity_manager: EntityManager,
    pub system_manager: SystemManager,
	pub custom_data: CustomWorldData,
    pub window: Window,
    pub default_camera_id: u16,
}

// Default world values
impl Default for World {
    fn default() -> Self {
        Self {
            component_manager: ComponentManager {
                current_component_id: 1,
                ..ComponentManager::default()
            },
            time_manager: Time::default(),
            input_manager: InputManager::default(),
            resource_manager: ResourceManager::default(),
            shader_manager: ShaderManager::default(),
            entity_manager: EntityManager::default(),
            texture_manager: TextureManager::default(),
            system_manager: SystemManager::default(),
			custom_data: CustomWorldData::default(),
            default_camera_id: 0,
            window: Window::default(),
        }
    }
}

impl World {
    // When the world started initializing
    pub fn start_world(&mut self, window: &mut glfw::Window) {
        // Load all the default things
        self.input_manager.setup_default_bindings();
        self.window.size = Self::get_default_window_size();
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos(0.0, 0.0);
        // Test stuff
        /*
        self.component_manager.register_component::<Position>();
        let mut test_entity = Entity::new("Test Entity");
        test_entity.link_default_component::<Position>(&mut self.component_manager);
        let entity_id = self.add_entity(test_entity);
        */
        register_components(self);
        load_systems(self);
        load_entities(self);
    }
    // We do the following in this function
    // 1. We update the entities of each UpdateSystem
    // 2. We tick the entities of each TickSystem (Only if the framecount is valid)
    // 3. We render the entities onto the screen using the RenderSystem
    pub fn update_world(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
        // Check for input events
        self.input_manager.update(window);
        // Check for default input events
        self.check_default_input_events(window, glfw);
        // Create the data for the systems
		let mut data: FireData = FireData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            input_manager: &mut self.input_manager,
            shader_manager: &mut self.shader_manager,
            texture_manager: &mut self.texture_manager,
            time_manager: &mut self.time_manager,
			resource_manager: &mut self.resource_manager,
			custom_data: &mut self.custom_data,
        };

        // Update the entities
        self.system_manager
            .run_system_type(SystemType::Update, &mut data);
        // And render them
        self.system_manager
            .run_system_type(SystemType::Render, &mut data);
        window.swap_buffers();

        // Update the timings of every system
        self.system_manager.update_systems(&self.time_manager);

        // Update the inputs
        self.input_manager
            .late_update(self.time_manager.delta_time as f32);
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
        // Change the debug view
        if self.input_manager.map_pressed("change_debug_view") {}
    }
    // Toggle fullscreen
    pub fn toggle_fullscreen(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
        self.window.fullscreen = !self.window.fullscreen;
        if self.window.fullscreen {
            // Set the glfw window as a fullscreen window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let videomode = monitor.unwrap().get_video_mode().unwrap();
                window.set_monitor(
                    glfw::WindowMode::FullScreen(monitor.unwrap()),
                    0,
                    0,
                    videomode.width,
                    videomode.height,
                    None,
                );
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(0, 0, videomode.width as i32, videomode.height as i32);
                }
            });
        } else {
            // Set the glfw window as a windowed window
            glfw.with_primary_monitor_mut(|_glfw2, monitor| {
                let _videomode = monitor.unwrap().get_video_mode().unwrap();
                let default_window_size = Self::get_default_window_size();
                window.set_monitor(
                    glfw::WindowMode::Windowed,
                    50,
                    50,
                    default_window_size.0 as u32,
                    default_window_size.1 as u32,
                    None,
                );
                unsafe {
                    // Update the OpenGL viewport
                    gl::Viewport(
                        0,
                        0,
                        default_window_size.0 as i32,
                        default_window_size.1 as i32,
                    );
                }
            });
        }
    }
    // When we want to close the application
    pub fn kill_world(&mut self) {
        let mut data: FireDataFragment = FireDataFragment {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
        };
        self.system_manager.kill_systems(&mut data);
    }
}

// Impl block for the entity stuff
impl World {
    // Wrapper function around the entity manager's add_entity
    pub fn add_entity(&mut self, entity: Entity) -> u16 {
        let id = self.entity_manager.add_entity(entity.clone());
        let entity = self.entity_manager.get_entity(id).clone();
        // Since we cloned the entity variable we gotta update the entity manager with the new one
        *self.entity_manager.get_entity_mut(id) = entity;
        return id;
    }
    // Wrapper function around the entity manager remove_entity
    pub fn remove_entity(&mut self, entity_id: u16) {
        // Remove the entity from the world first
        let removed_entity = self.entity_manager.remove_entity(entity_id);
    }
    // Get a mutable reference to an entity from the entity manager
    pub fn get_entity_mut(&mut self, entity_id: u16) -> &mut Entity {
        self.entity_manager.get_entity_mut(entity_id)
    }
    // Get a reference to an entity from the entity manager
    pub fn get_entity(&self, entity_id: u16) -> &Entity {
        self.entity_manager.get_entity(entity_id)
    }
}

// Impl block related to the windowing / rendering stuff
impl World {
    // Get the default width and height of the starting window
    pub fn get_default_window_size() -> (i32, i32) {
        (1280, 720)
    }
    // When we resize the window
    pub fn resize_window_event(&mut self, size: (i32, i32)) {
        unsafe {
            gl::Viewport(0, 0, size.0, size.1);
            /*
            let system_component: &mut RendererS = self
                .component_manager
                .system_components
                .get_mut(self.window.system_renderer_component_index as usize)
                .unwrap()
                .as_any_mut()
                .downcast_mut()
                .unwrap();
            // Update the size of each texture that is bound to the framebuffer
            system_component
                .color_texture
                .update_size(size.0 as u32, size.1 as u32);
            system_component
                .depth_stencil_texture
                .update_size(size.0 as u32, size.1 as u32);
            system_component
                .normals_texture
                .update_size(size.0 as u32, size.1 as u32);
            system_component
                .position_texture
                .update_size(size.0 as u32, size.1 as u32);
            */
        }
        let camera_entity_clone = self.get_entity(self.default_camera_id).clone();
        let entity_clone_id = camera_entity_clone.entity_id;
        let camera_component =
            camera_entity_clone.get_component_mut::<Camera>(&mut self.component_manager);
        camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
        camera_component.window_size = size;
        camera_component.update_projection_matrix();
        // Update the original entity
        *self.get_entity_mut(entity_clone_id) = camera_entity_clone;
        self.window.size = size;
    }
}

// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
	pub main_camera_entity_id: u16
}
// Static time variables
#[derive(Default)]
pub struct Time {
    pub time_since_start: f64,
    pub delta_time: f64,
}
