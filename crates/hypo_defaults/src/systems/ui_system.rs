use std::ffi::c_void;

use super::super::components;
use hypo_ecs::{Entity, FilteredLinkedComponents};
use hypo_input::*;
use hypo_system_event_data::{SystemEventData, SystemEventDataLite};
use hypo_systems::{System, SystemData};
#[derive(Default)]
pub struct UISystem {
    pub system_data: SystemData,
    pub vertex_arrays: Vec<u32>,
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
        let system_data = self.get_system_data_mut();
        unsafe {
            let mut vertex_array: u32 = 0;
            gl::GenVertexArrays(1, &mut vertex_array as *mut u32);
            gl::BindVertexArray(vertex_array);
            let mut vertex_buffer: u32 = 0;
            gl::GenBuffers(1, &mut vertex_buffer as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            let vertices: Vec<f32> = vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0];
            gl::BufferData(gl::ARRAY_BUFFER, 3 * 3, vertices.as_ptr() as *const c_void, gl::STATIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, vertices.as_ptr() as *const c_void);
            self.vertex_arrays.push(vertex_array);
        }
    }

    // Called for each entity in the system
    fn fire_entity(&mut self, components: &FilteredLinkedComponents, data: &mut SystemEventData) {}

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
        unsafe {
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
