use glfw::Context;



use crate::engine::core::ecs::component::*;
use crate::engine::core::ecs::entity::*;
use crate::engine::core::ecs::system::*;
use crate::engine::core::ecs::system_data::*;
use crate::engine::input::*;
use crate::engine::rendering::*;

use crate::engine::core::defaults::components::components::Camera;
use crate::engine::rendering::shader::Shader;
use crate::engine::rendering::shader::SubShader;
use crate::engine::rendering::texture::Texture;
use crate::engine::resources::ResourceManager;

use crate::game::level::*;
use crate::engine::core::ecs::error::ECSError;

// Import stuff from the rendering module

use super::cacher::CacheManager;

use super::defaults::systems::rendering_system::RenderingSystem;
use super::ecs::entity_manager::EntityManager;

//  The actual world
#[derive(Default)]
pub struct World {
    // Managers
    pub component_manager: ComponentManager,
    pub input_manager: InputManager,
    pub resource_manager: ResourceManager,
    pub texture_cacher: CacheManager<Texture>,
    // Shaders
    pub shader_cacher: (CacheManager<SubShader>, CacheManager<Shader>),
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
        self.input_manager.setup_default_bindings();
        self.custom_data.window.size = Self::get_default_window_size();
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos(0.0, 0.0);

        // Load the default objects for the CacheManagers
        let _white_texture = Texture::new()
            .load_texture(
                "textures\\white.png",
                &mut self.resource_manager,
                &mut self.texture_cacher,
            )
            .unwrap();
        let _black_texture = Texture::new()
            .load_texture(
                "textures\\black.png",
                &mut self.resource_manager,
                &mut self.texture_cacher,
            )
            .unwrap();
        self.texture_cacher
            .generate_defaults(vec!["textures\\white.png", "textures\\black.png"]);

        // Copy the default shader name
        let default_shader_name: String;
        {
            let default_shader = Shader::new(
                vec!["shaders\\default.vrsh.glsl", "shaders\\default.frsh.glsl"],
                &mut self.resource_manager,
                &mut self.shader_cacher,
            );
            default_shader_name = default_shader.1;
        }
        self.shader_cacher
            .1
            .generate_defaults(vec![default_shader_name.as_str()]);
    }
    // When the world started initializing
    pub fn start_world(&mut self, window: &mut glfw::Window) {
        // Load the default stuff
        self.load_defaults(window);
        // Test stuff
        /*
        self.component_manager.register_component::<Position>();
        let mut test_entity = Entity::new("Test Entity");
        test_entity.link_default_component::<Position>(&mut self.component_manager);
        let entity_id = self.add_entity(test_entity);
        */
        register_components(self);
        load_systems(self);
        // Add the entities that need to be added
        self.add_entities(self.entity_manager.entitites_to_add.clone());
        // So we don't cause an infinite loop lol
        self.entity_manager.entitites_to_add.clear();

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
        let mut data: SystemEventData = SystemEventData {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
            input_manager: &mut self.input_manager,
            shader_cacher: &mut self.shader_cacher,
            texture_cacher: &mut self.texture_cacher,
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

        // Add the entities that need to be added
        self.add_entities(self.entity_manager.entitites_to_add.clone());
        // So we don't cause an infinite loop lol
        self.entity_manager.entitites_to_add.clear();
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
        // Capture the fps
        if self.input_manager.map_pressed("capture_fps") {
            println!(
                "Current FPS: '{}', Delta: '{}'",
                self.time_manager.fps, self.time_manager.delta_time
            );
        }
        // Change the debug view
        if self.input_manager.map_pressed("change_debug_view") {
            let render_system = self
                .system_manager
                .get_system_mut(self.custom_data.render_system_id)
                .as_any_mut()
                .downcast_mut::<RenderingSystem>()
                .unwrap();
            render_system.debug_view += 1;
            render_system.debug_view %= 4;
        }
		// Enable / Disable wireframe
		if self.input_manager.map_pressed("toggle_wireframe") {
			let render_system = self
                .system_manager
                .get_system_mut(self.custom_data.render_system_id)
                .as_any_mut()
                .downcast_mut::<RenderingSystem>()
                .unwrap();
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
        let mut data: SystemEventDataLite = SystemEventDataLite {
            entity_manager: &mut self.entity_manager,
            component_manager: &mut self.component_manager,
			custom_data: &mut self.custom_data
        };
        self.system_manager.kill_systems(&mut data);
    }
}

// Impl block for the entity stuff
impl World {
    // Wrapper function around the entity manager's add_entity
    pub fn add_entity(&mut self, entity: Entity) -> u16 {
        let id = self.entity_manager.internal_add_entity(entity);
        let entity = self.entity_manager.get_entity(id).unwrap().clone();
        // Since we cloned the entity variable we gotta update the entity manager with the new one
        self.system_manager.add_entity_to_systems(
            &entity,
            &mut SystemEventDataLite {
                entity_manager: &mut self.entity_manager,
                component_manager: &mut self.component_manager,
				custom_data: &mut self.custom_data
            },
        );
		println!("{:?}", entity);
        *self.entity_manager.get_entity_mut(id).unwrap() = entity;
        id
    }
    // Add multiple entities at once
    pub fn add_entities(&mut self, entities: Vec<Entity>) -> Vec<u16> {
        let mut result: Vec<u16> = Vec::new();
        // Add all the entities
        for entity in entities {
            result.push(self.add_entity(entity));
        }
        result
    }
    // Wrapper function around the entity manager remove_entity
    pub fn remove_entity(
        &mut self,
        entity_id: u16,
    ) -> Result<Entity, ECSError> {
        // Remove the entity from the world first
        let removed_entity = self.entity_manager.remove_entity(entity_id)?;
        // Remove all the components this entity had
        for (_, global_id) in removed_entity.components.iter() {
            self.component_manager.remove_component(*global_id);
        }
        Ok(removed_entity)
    }
    // Remove multiple entities at once
    pub fn remove_entities(&mut self, entity_ids: Vec<u16>) -> Vec<Entity> {
        let mut result: Vec<Entity> = Vec::new();
        // Remove the specified entities
        for entity_id in entity_ids {
            result.push(self.remove_entity(entity_id).unwrap());
        }
        result
    }
}

// Impl block related to the windowing / rendering stuff
impl World {
    // Get the default width and height of the starting window
    pub fn get_default_window_size() -> (u16, u16) {
        (1280, 720)
    }
    // When we resize the window
    pub fn resize_window_event(&mut self, size: (u16, u16)) {
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);

            let render_system = self
                .system_manager
                .get_system_mut(0)
                .as_any_mut()
                .downcast_mut::<RenderingSystem>()
                .unwrap();
            // Update the size of each texture that is bound to the framebuffer
			render_system.window.size = size;
            render_system.diffuse_texture.update_size(size.0, size.1);
            render_system
                .depth_stencil_texture
                .update_size(size.0, size.1);
            render_system.normals_texture.update_size(size.0, size.1);
            render_system.position_texture.update_size(size.0, size.1);
            render_system.emissive_texture.update_size(size.0, size.1);
        }
        let camera_entity_clone = self
            .entity_manager
            .get_entity(self.custom_data.main_camera_entity_id)
            .unwrap()
            .clone();
        let entity_clone_id = camera_entity_clone.entity_id;
        let camera_component = camera_entity_clone
            .get_component_mut::<Camera>(&mut self.component_manager)
            .unwrap();
        camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
        camera_component.update_projection_matrix(&self.custom_data.window);
        // Update the original entity
        *self.entity_manager.get_entity_mut(entity_clone_id).unwrap() = camera_entity_clone;
        self.custom_data.window.size = size;
    }
}

// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
    pub main_camera_entity_id: u16,
    pub sky_component_id: u16,
    pub render_system_id: u8,
    pub sun_rotation: glam::Quat,
	pub window: Window,
}
// Static time variables
#[derive(Default)]
pub struct Time {
    pub seconds_since_game_start: f64,
    pub delta_time: f64,
	pub fps: f64
}
