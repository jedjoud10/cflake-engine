use std::{ffi::c_void, ptr::null};
use core::mem::size_of;

use super::super::components;
use hypo_ecs::{Entity, FilteredLinkedComponents};
use hypo_input::*;
use hypo_rendering::Shader;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::{System, SystemData, SystemFiringType};
use hypo_ui::{Element, ElementType};
#[derive(Default)]
pub struct UISystem {
    pub system_data: SystemData,
    pub ui_shader_name: String,
    pub verts: Vec<veclib::Vector2<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub vertex_array: u32,
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
            gl::BufferData(gl::ARRAY_BUFFER, (self.verts.len() * size_of::<f32>() * 2) as isize, self.verts.as_ptr() as *const c_void, gl::STATIC_DRAW);
            // Enable the vertex attrib array 0 (vertex buffer)
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, null());

            gl::GenBuffers(1, &mut uv_buffer as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, uv_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, (self.uvs.len() * size_of::<f32>() * 2) as isize, self.uvs.as_ptr() as *const c_void, gl::STATIC_DRAW);
            // Enable the vertex attrib array 1 (uv buffer)
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, uv_buffer);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, null());
            self.vertex_array = vertex_array;
        }
        let root = &mut data.ui_manager.root;
        // Load the UI shader
        let shader_name = Shader::new(vec!["defaults\\shaders\\ui_elem.vrsh.glsl", "defaults\\shaders\\ui_panel.frsh.glsl"], data.resource_manager, data.shader_cacher).1;
        self.ui_shader_name = shader_name;
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, _components: &FilteredLinkedComponents, _data: &mut SystemEventData) { println!("{}", _components.entity_id) }

    // Render all the elements onto the screen
    fn post_fire(&mut self, data: &mut SystemEventData) {
        // Draw each element, from back to front
        let elements = data
            .ui_manager
            .root
            .smart_element_list
            .elements
            .iter()
            .filter_map(|x| x.as_ref())
            .collect::<Vec<&hypo_ui::Element>>();
        let shader = data.shader_cacher.1.get_object(&self.ui_shader_name).unwrap();         
        let root = &data.ui_manager.root;

        // Draw every element
        /*
        for element in elements {
            shader.use_shader(); 
            unsafe {            
                gl::Disable(gl::CULL_FACE);
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                // Always remember to clear the depth buffer
                gl::Clear(gl::DEPTH_BUFFER_BIT);
                gl::BindVertexArray(self.vertex_array);
                // Update the shader uniforms
                let depth = (1.0 - (element.depth as f32 / root.max_depth as f32)) * 0.99;
                shader.set_f32("depth", &depth);
                shader.set_vec2f32("size", &element.size);
                shader.set_vec2f32("position", &element.position);
                shader.set_vec3f32("color", &element.color);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);            
            } 
        }
        */
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
