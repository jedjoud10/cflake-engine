use core::mem::size_of;
use std::{ffi::c_void, ptr::null};

use super::super::components;
use ecs::{Entity, FilteredLinkedComponents};
use fonts::Font;
use input::*;
use rendering::Shader;
use resources::LoadableResource;
use system_event_data::{SystemEventData, SystemEventDataLite};
use systems::{System, SystemData, SystemFiringType};
use ui::{CoordinateType, Element, ElementType, Root};
#[derive(Default)]
pub struct UISystem {
    pub system_data: SystemData,
    pub ui_shader_name: String,
    pub font_ui_shader_name: String,
    pub verts: Vec<veclib::Vector2<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub vertex_array: u32,
}

// Draw functions
impl UISystem {
    // Set the default shader arguments to draw a normal panel
    fn set_default_draw_arguments(&self, element_data: (veclib::Vector2<f32>, veclib::Vector2<f32>, veclib::Vector4<f32>, f32), shader: &Shader) {
        // Update the shader arguments
        shader.set_f32("depth", &element_data.3);
        shader.set_vec2f32("size", &element_data.1);
        shader.set_vec2f32("offset_position", &element_data.0);
        shader.set_vec4f32("color", &element_data.2);
    }
    // Draw the panel vertices
    fn draw_panel_vertices(&self) {
        unsafe {
            // Draw the element
            gl::BindVertexArray(self.vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
    // Draw a simple panel to the screen
    fn draw_panel(&self, element_data: (veclib::Vector2<f32>, veclib::Vector2<f32>, veclib::Vector4<f32>, f32), shader: &Shader) {
        self.set_default_draw_arguments(element_data, shader);
        self.draw_panel_vertices();
    }
    // Draw the text by drawing multiple elements
    fn draw_text(&self, element_data: (veclib::Vector2<f32>, veclib::Vector2<f32>, veclib::Vector4<f32>, f32), shader: &Shader, text_content: &String, font: &Font) {
        // Draw each character in the string as a separate element
        let chars = font.convert_text_to_font_chars(text_content);        
        for char in chars {
            // Set the default panel arguments
            self.set_default_draw_arguments(element_data, shader);
            // Set the atlas texture and the character padding
            shader.set_t2d("atlas_texture", font.texture.as_ref().unwrap(), gl::TEXTURE0);
            // Set the font char padding in the shader
            let min_padding: veclib::Vector2<f32> = char.min.into();
            let max_padding: veclib::Vector2<f32> = char.max.into();
            shader.set_vec2f32("min_padding", &veclib::Vector2::ZERO);
            shader.set_vec2f32("max_padding", &(veclib::Vector2::ONE));
            shader.set_vec2f32("min_padding", &(min_padding / 512.0));
            shader.set_vec2f32("max_padding", &(max_padding / 512.0));
            // Draw each character as panel
            self.draw_panel_vertices();
        }                
    }
}

// The UI system which is going to render the elements and handle UI input for the elements
impl System for UISystem {
    // Wrappers around system data
    fn get_system_data(&self) -> &SystemData {
        &self.system_data
    }

    fn get_system_data_mut(&mut self) -> &mut SystemData {
        &mut self.system_data
    }

    // Setup the system
    fn setup_system(&mut self, data: &mut SystemEventData) {
        // Set the UI system stuff
        {
            let data = self.get_system_data_mut();
            data.firing_type = SystemFiringType::OnlySystems;
        }
        // Load the vertices and UVs for a simple quad
        self.verts = vec![
            // First triangle
            veclib::Vector2::new(-1.0, -1.0),
            veclib::Vector2::new(-1.0, 1.0),
            veclib::Vector2::new(1.0, 1.0),
            // Second triangle
            veclib::Vector2::new(1.0, 1.0),
            veclib::Vector2::new(1.0, -1.0),
            veclib::Vector2::new(-1.0, -1.0),
        ];
        self.uvs = vec![
            // First triangle
            veclib::Vector2::new(0.0, 0.0),
            veclib::Vector2::new(0.0, 1.0),
            veclib::Vector2::new(1.0, 1.0),
            // Second triangle
            veclib::Vector2::new(1.0, 1.0),
            veclib::Vector2::new(1.0, 0.0),
            veclib::Vector2::new(0.0, 0.0),
        ];
        unsafe {
            // Create the vertex array
            let mut vertex_array: u32 = 0;
            gl::GenVertexArrays(1, &mut vertex_array as *mut u32);
            gl::BindVertexArray(vertex_array);
            // The vertex buffer
            let mut vertex_buffer: u32 = 0;
            // The uv buffer
            let mut uv_buffer: u32 = 0;
            gl::GenBuffers(1, &mut vertex_buffer as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.verts.len() * size_of::<f32>() * 2) as isize,
                self.verts.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            // Enable the vertex attrib array 0 (vertex buffer)
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, null());

            gl::GenBuffers(1, &mut uv_buffer as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, uv_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.uvs.len() * size_of::<f32>() * 2) as isize,
                self.uvs.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            // Enable the vertex attrib array 1 (uv buffer)
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, uv_buffer);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, null());
            self.vertex_array = vertex_array;
        }
        // Load a default font that we can use for testing
        let default_font = Font::new().from_path("defaults\\fonts\\default_font.font", data.resource_manager).unwrap();
        // Set the default font
        data.ui_manager.font_manager.add_font(default_font);

        // Create the default UI, just a blank screen
        let mut root = Root::new();
        let root_elem = Element::default();
        // Add the element to the root node
        root.add_element(root_elem);
        // ----Add the elements here----

        // Create a text element
        let text_element = Element::new()
            .set_coordinate_system(CoordinateType::Pixel)
            .set_position(veclib::Vector2::ZERO)
            .set_size(veclib::Vector2::ONE * 1000.0)
            .set_text("Tomato");
        root.add_element(text_element);

        // Set this as the default root
        data.ui_manager.set_default_root(root);
        // Load the UI shader
        self.ui_shader_name = Shader::new(
            vec!["defaults\\shaders\\ui_elem.vrsh.glsl", "defaults\\shaders\\ui_panel.frsh.glsl"],
            data.resource_manager,
            data.shader_cacher,
            None,
        )
        .1;
        // Load the UI font shader
        self.font_ui_shader_name = Shader::new(
            vec!["defaults\\shaders\\ui_font.vrsh.glsl", "defaults\\shaders\\ui_font.frsh.glsl"],
            data.resource_manager,
            data.shader_cacher,
            None,
        )
        .1;
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) {
    }

    // Render all the elements onto the screen
    fn post_fire(&mut self, data: &mut SystemEventData) {
        // Draw each element, from back to front
        let root = data.ui_manager.get_default_root();
        let elements = root.smart_element_list.elements.iter().filter_map(|x| x.as_ref()).collect::<Vec<&ui::Element>>();
        let shader = data.shader_cacher.1.get_object(&self.ui_shader_name).unwrap();
        // Get the font shader
        let font_shader = data.shader_cacher.1.get_object(&self.font_ui_shader_name).unwrap();
        // Default font
        let default_font = data.ui_manager.font_manager.get_font("defaults\\fonts\\default_font.font");

        // Draw every element, other than the root element
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            // Always remember to clear the depth buffer
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            // Enable transparency only for the UI elements
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        for element in elements {
            let bad_element_type = match element.element_type {
                ElementType::Empty => true,
                _ => false,
            };
            if element.id == 0 || bad_element_type {
                continue;
            }
            // Get the data that will be passed to the shader
            let depth = (1.0 - (element.depth as f32 / root.max_depth as f32)) * 0.99;
            let size: veclib::Vector2<f32>;
            let position: veclib::Vector2<f32>;
            let resolution = veclib::Vector2::<f32>::from(data.custom_data.window.size);
            match element.coordinate_type {
                ui::CoordinateType::Pixel => {
                    // Pixel coordinate type
                    size = element.size / resolution;
                    position = element.position / resolution;
                }
                ui::CoordinateType::Factor => {
                    // Factor coordinate type
                    size = element.size;
                    position = element.position;
                }
            }
            // Create da tuple
            let tuple = (position, size, element.color, depth);

            // Every type that isn't the text type
            match &element.element_type {
                ElementType::Text(text_content) => {
                    // Use the font shader
                    font_shader.use_shader();
                    // Draw the text
                    self.draw_text(tuple, &font_shader, text_content, default_font);
                }
                _ => {
                    // Use the normal panel shader
                    shader.use_shader();
                    // Draw the panel
                    self.draw_panel(tuple, &shader);
                }
            }
        }
        // Disable transparency after drawing the ui elements
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
