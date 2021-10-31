use std::{ffi::c_void, mem::size_of, ptr::null};

use assets::Asset;

// A simple model that holds vertex, normal, and color data
#[derive(Default, Debug, Clone)]
pub struct Model {
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub triangles: Vec<u32>,
}

impl Asset for Model {
    // Load a model from an asset
    fn asset_load(data: &assets::AssetMetadata) -> Self where Self: Sized {
        
    }
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
        return output_model;
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
        return output_model;
    }
    // Create some GPU data from this specific model
    pub fn refresh_gpu_data(&self) -> ModelDataGPU {
        let mut gpu_data = ModelDataGPU::default();
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut gpu_data.vertex_array_object);
            gl::BindVertexArray(gpu_data.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut gpu_data.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, gpu_data.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.triangles.len() * size_of::<u32>()) as isize,
                self.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut gpu_data.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * size_of::<f32>() * 3) as isize,
                self.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut gpu_data.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.normals.len() * size_of::<f32>() * 3) as isize,
                self.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // And it's brother, the tangent buffer
            gl::GenBuffers(1, &mut gpu_data.tangent_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.tangents.len() * size_of::<f32>() * 4) as isize,
                self.tangents.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // The texture coordinates buffer
            gl::GenBuffers(1, &mut gpu_data.uv_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.uvs.len() * size_of::<f32>() * 2) as isize,
                self.uvs.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            // Finally, the vertex colors buffer
            gl::GenBuffers(1, &mut gpu_data.color_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.colors.len() * size_of::<f32>() * 3) as isize,
                self.colors.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Tangent attribute
            gl::EnableVertexAttribArray(2);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.tangent_buf);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());

            // UV attribute
            gl::EnableVertexAttribArray(3);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.uv_buf);
            gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());

            // Vertex color attribute
            gl::EnableVertexAttribArray(4);
            gl::BindBuffer(gl::ARRAY_BUFFER, gpu_data.color_buf);
            gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());

            gpu_data.initialized = true;
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        gpu_data
    }
}

// Struct that hold the model's information from OpenGL
#[derive(Default, Clone)]
pub struct ModelDataGPU {
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub uv_buf: u32,
    pub tangent_buf: u32,
    pub color_buf: u32,
    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub initialized: bool,
}

impl ModelDataGPU {
    // Dipose
    pub fn dispose(&mut self) {
        unsafe {
            if self.initialized {
                // Delete the VBOs
                gl::DeleteBuffers(1, &mut self.vertex_buf);
                gl::DeleteBuffers(1, &mut self.normal_buf);
                gl::DeleteBuffers(1, &mut self.uv_buf);
                gl::DeleteBuffers(1, &mut self.tangent_buf);
                gl::DeleteBuffers(1, &mut self.color_buf);
                gl::DeleteBuffers(1, &mut self.element_buffer_object);

                // Delete the vertex array
                gl::DeleteVertexArrays(1, &mut self.vertex_array_object);
                self.initialized = false;
                errors::ErrorCatcher::catch_opengl_errors();
            }
        }
    }
}

// Trait that allows you to make procedural models
pub trait ProceduralModelGenerator {
    // Generate the model
    fn generate_model(&self) -> Model;
}
