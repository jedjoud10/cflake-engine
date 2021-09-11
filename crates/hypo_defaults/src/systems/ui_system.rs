use std::{ffi::c_void, ptr::null};
use core::mem::size_of;

use super::super::components;
use hypo_ecs::{Entity, FilteredLinkedComponents};
use hypo_input::*;
use hypo_rendering::Shader;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::{System, SystemData, SystemFiringType};
#[derive(Default)]
pub struct UISystem {
    pub system_data: SystemData,
    pub vertex_arrays: Vec<u32>,
    pub ui_shader_name: String,
    panel_verts: Vec<f32>,
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
        {
            let data = self.get_system_data_mut();
            data.firing_type = SystemFiringType::OnlySystems;
        }
        unsafe {
            self.panel_verts = vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0];
            let mut vertex_array: u32 = 0;
            gl::GenVertexArrays(1, &mut vertex_array as *mut u32);
            gl::BindVertexArray(vertex_array);
            let mut vertex_buffer: u32 = 0;
            gl::GenBuffers(1, &mut vertex_buffer as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER, (self.panel_verts.len() * size_of::<f32>()) as isize, self.panel_verts.as_ptr() as *const c_void, gl::STATIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
            self.vertex_arrays.push(vertex_array);
        }
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
        shader.use_shader();
        unsafe {            
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(self.vertex_arrays[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);            
        }
        for element in elements {}
    }

    // Turn this into "Any" so we can cast into child systems
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
