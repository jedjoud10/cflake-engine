use std::{ffi::c_void, fmt::Display, mem::size_of, ptr::null};

// A simple model that holds vertex, normal, and color data
#[derive(Default, Clone)]
pub struct Model {
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub triangles: Vec<u32>,
}

impl Model {
    // Create a new empty model
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            triangles: Vec::new(),
        }
    }
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    pub fn combine(&self, other: &Self) -> Self {
        let mut output_model = self.clone();
        let max_triangle_index: u32 = self.vertices.len() as u32;
        // Get the max triangle inde
        let mut final_tris = other.triangles.clone();
        for x in final_tris.iter_mut() {
            *x += max_triangle_index;
        }
        output_model.triangles.extend(final_tris);
        output_model.vertices.extend(other.vertices.clone());
        output_model.normals.extend(other.normals.clone());
        output_model.uvs.extend(other.uvs.clone());
        output_model.colors.extend(other.colors.clone());
        output_model.tangents.extend(other.tangents.clone());
        output_model
    }
    // Comebine a model with this one
    // NOTE: This assumes that the second model uses vertices from the first model
    pub fn combine_smart(&self, other: &Self) -> Self {
        let mut output_model: Self = self.clone();
        output_model.triangles.extend(other.triangles.clone());
        output_model.vertices.extend(other.vertices.clone());
        output_model.normals.extend(other.normals.clone());
        output_model.uvs.extend(other.uvs.clone());
        output_model.colors.extend(other.colors.clone());
        output_model.tangents.extend(other.tangents.clone());
        output_model
    }
}
