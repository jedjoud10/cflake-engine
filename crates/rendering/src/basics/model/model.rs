use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
};
use std::{ffi::c_void, mem::size_of, ptr::null};

use super::VertexAttributeBufferLayout;

// A simple model that holds vertex, normal, and color data
pub struct Model {
    // Main IDs
    pub vertex_array_object: u32,

    // Vertex attributes IDs
    pub buffers: [u32; 6],
    /*
    pub element_buffer_object: u32,

    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,

    pub color_buf: u32,
    pub uv_buf: u32,
    */
    // Vertex attribute arrays
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<i8>>,
    pub tangents: Vec<veclib::Vector4<i8>>,
    pub uvs: Vec<veclib::Vector2<u8>>,
    pub colors: Vec<veclib::Vector3<u8>>,

    // How we set the VBO buffers
    pub layout: VertexAttributeBufferLayout,

    // Triangles
    pub triangles: Vec<u32>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            vertex_array_object: 0,
            buffers: [0; 6],
            vertices: Default::default(),
            normals: Default::default(),
            tangents: Default::default(),
            uvs: Default::default(),
            colors: Default::default(),
            layout: Default::default(),
            triangles: Default::default(),
        }
    }
}

impl PipelineObject for Model {
    // Reserve an ID for this model
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.models.get_next_id_increment())))
    }
    // Send this model to the pipeline for construction
    fn send(self, _pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Model(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(_pipeline: &Pipeline, id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Model(Deconstruct::<Self>(id))
    }
    // Add the model to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // No wasted space
        self.vertices.shrink_to_fit();
        self.triangles.shrink_to_fit();
        self.colors.shrink_to_fit();
        self.tangents.shrink_to_fit();
        self.uvs.shrink_to_fit();
        // Add the model
        unsafe {
            // We simply don't have any vertices to render

            // Create the VAO
            gl::GenVertexArrays(1, &mut self.vertex_array_object);
            gl::BindVertexArray(self.vertex_array_object);


            // We can create all the buffers at once
            let mut buffers = [0_32; 6];
            gl::GenBuffers(6, buffers.as_mut_ptr());

            // Create the EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[0]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.triangles.len() * size_of::<u32>()) as isize,
                self.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // We now have 2 ways to represent the vertex attributes: either we pack them tightly in their own VBO, or we interleave them
            if let VertexAttributeBufferLayout::Interleaved = self.layout {
                // Interleaved
                /*
                Interleaved VBO
                pub element_buffer_object: u32,
                pub vertex_buf: u32,

                pub tangent_buf: u32,
                pub normal_buf: u32,

                pub color_buf: u32,
                pub uv_buf: u32,
                */
            } else {
                // Normal, fallback
                // Create the vertex buffer and populate it
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[1]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * size_of::<f32>() * 3) as isize,
                    self.vertices.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // Vertex attrib array
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());
                

                // Vertex normals attribute
                if !self.normals.is_empty() {
                    // Vertex normals buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[2]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.normals.len() * size_of::<i8>() * 3) as isize,
                        self.normals.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );
                    
                    gl::EnableVertexAttribArray(1);
                    gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, 0, null());                    
                }

                if !self.tangents.is_empty() {
                    // And it's brother, the tangent buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[3]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.tangents.len() * size_of::<i8>() * 4) as isize,
                        self.tangents.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    // Tangent attribute
                    gl::EnableVertexAttribArray(2);
                    gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, 0, null());
                }

                if !self.uvs.is_empty() {
                    // The texture coordinates buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[4]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.uvs.len() * size_of::<u8>() * 2) as isize,
                        self.uvs.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    // UV attribute
                    gl::EnableVertexAttribArray(3);
                    gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                }

                
                if !self.colors.is_empty() {
                    // Vertex colors buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[5]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.colors.len() * size_of::<u8>() * 3) as isize,
                        self.colors.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );
                    
                    // Vertex colors attribute
                    gl::EnableVertexAttribArray(4);
                    gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                }
            }

            // Unbind
            self.buffers = buffers;
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Add the model
        pipeline.models.insert(id.get()?, self);
        Some(())
    }
    // Remove the model from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let model = pipeline.models.remove(id.get()?)?;
        // Dispose of the OpenGL buffers
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(model.buffers.len() as i32, model.buffers.as_ptr());

            // Delete the vertex array
            gl::DeleteVertexArrays(1, &model.vertex_array_object);
        }
        Some(model)
    }
}

impl Model {
    // Create the model with a specific vertex attribute vbo layout
    pub fn with_layout(mut self, layout: VertexAttributeBufferLayout) -> Self {
        self.layout = layout;
        self
    }
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    pub fn combine(mut self, other: Self) -> Self {
        let max_triangle_index: u32 = self.vertices.len() as u32;
        self.triangles.extend(other.triangles.into_iter().map(|mut x| {
            x += max_triangle_index;
            x
        }));
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        self
    }
    // Procedurally generate the normals for this model
    pub fn with_generated_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles

        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; self.vertices.len()];
        for i in 0..(self.triangles.len() / 3) {
            // Calculate the face normal
            let (i1, i2, i3) = (self.triangles[i * 3], self.triangles[i * 3 + 1], self.triangles[i * 3 + 2]);
            // Get the actual vertices
            let a = self.vertices.get(i1 as usize).unwrap();
            let b = self.vertices.get(i2 as usize).unwrap();
            let c = self.vertices.get(i3 as usize).unwrap();

            // Calculate
            let d1 = b - a;
            let d2 = c - a;
            let cross = veclib::Vector3::<f32>::cross(d1, d2).normalized();

            // Add the face normal to our local vertices
            vertex_normals[i1 as usize] += cross;
            vertex_normals[i2 as usize] += cross;
            vertex_normals[i3 as usize] += cross;
        }

        // Now we must normalize
        for vertex_normal in vertex_normals.iter_mut() {
            vertex_normal.normalize();
        }

        // Update our normals
        self.normals = vertex_normals.into_iter().map(|x| (x * 127.0).normalized().into()).collect::<Vec<_>>();
    }
}
