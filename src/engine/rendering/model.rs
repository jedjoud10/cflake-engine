use crate::engine::core::world::World;
use crate::engine::resources::Resource;
use gl;
use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    mem::size_of,
    ptr::null,
};

// A simple model that holds vertex, normal, and color data
#[derive(Default, Debug)]
pub struct Model {
    pub vertices: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub tangents: Vec<glam::Vec4>,
    pub uvs: Vec<glam::Vec2>,
    pub triangles: Vec<u32>,
}

impl Model {
    // Turns a loaded resource model into an actual model
    pub fn from_resource(resource: &Resource) -> Option<Self> {
        match resource {
            Resource::Model(model) => {
                // Turn the loaded model into a normal model
                let new_model = Self {
                    vertices: model.vertices.clone(),
                    normals: model.normals.clone(),
                    tangents: model.tangents.clone(),
                    uvs: model.uvs.clone(),
                    triangles: model.indices.clone(),
                };
                return Some(new_model);
            }
            _ => return None,
        }
    }
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            let copy = self.triangles[i];
            self.triangles[i] = self.triangles[i + 2];
            self.triangles[i + 2] = copy;
        }
    }
}

// Struct that hold the model's information from OpenGL
#[derive(Default)]
pub struct ModelDataGPU {
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub uv_buf: u32,
    pub tangent_buf: u32,
    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub initialized: bool,
    pub model_matrix: glam::Mat4,
}

// Trait that allows you to make procedural models
pub trait ProceduralModelGenerator {
    // Generate the model
    fn generate_model(&self) -> Model;
}
