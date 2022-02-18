use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
};
use std::{ffi::c_void, mem::size_of, ptr::null};
use gl::types::GLuint;
use super::{VertexAttributeBufferLayout, VertexBuilder, Vertices};

// A simple model that holds vertex, normal, and color data
#[derive(Default)]
pub struct Model {
    // Main IDs
    pub vertex_array_object: GLuint,

    // Vertex attributes IDs
    pub buffers: [GLuint; 6],
    /*
    pub element_buffer_object: u32,

    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,

    pub color_buf: u32,
    pub uv_buf: u32,
    */
    // Store the vertices (in multiple bufer or in a single big buffer)
    pub vertices: Vertices,

    // How we set the VBO buffers
    pub layout: VertexAttributeBufferLayout,

    // Triangles
    pub triangles: Vec<u32>,
}

impl PipelineObject for Model {
    // Reserve an ID for this model
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.models.gen_id()))
    }
    // Send this model to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Model(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Model(Deconstruct::<Self>(id))
    }
    // Add the model to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the model
        unsafe {
            // We simply don't have any vertices to render

            // Create the VAO
            gl::GenVertexArrays(1, &mut self.vertex_array_object);
            gl::BindVertexArray(self.vertex_array_object);

            // We can create all the buffers at once
            let mut buffers = [0_u32; 6];
            gl::GenBuffers(1, buffers.as_mut_ptr());

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
                // Calculate the stride
                let mut stride;
                // Positions
                stride = size_of::<f32>() * 3;
                // Normals (optional)
                if !self.vertices.normals.is_empty() { stride += size_of::<i8>() * 3; }
                // Tangents (optional)
                if !self.vertices.tangents.is_empty() { stride += size_of::<i8>() * 4; }
                // UVS (optional)
                if !self.vertices.uvs.is_empty() { stride += size_of::<u8>() * 2; }
                // Colors (optional)
                if !self.vertices.colors.is_empty() { stride += size_of::<u8>() * 3; }
                let stride = stride as i32;

                // Convert the different vectors into a big chunky one
                let mut big_vector = Vec::with_capacity(self.vertices.len() * stride as usize);
                // Kill me.
                // "I must optimize this but that is a problem for future me" - Me atm
                for (idx, x) in self.vertices.positions.iter().enumerate() {
                    // Add
                    big_vector.extend_from_slice(&std::mem::transmute_copy::<veclib::Vector3<f32>, [u8; size_of::<f32>() * 3]>(x));
                    if !self.vertices.normals.is_empty() { 
                        big_vector.extend_from_slice(&std::mem::transmute_copy::<veclib::Vector3<i8>, [u8; size_of::<i8>() * 3]>(self.vertices.normals.get(idx)?));
                    }
                    if !self.vertices.tangents.is_empty() { 
                        big_vector.extend_from_slice(&std::mem::transmute_copy::<veclib::Vector4<i8>, [u8; size_of::<i8>() * 4]>(self.vertices.tangents.get(idx)?));
                    }
                    if !self.vertices.uvs.is_empty() { 
                        big_vector.extend_from_slice(&std::mem::transmute_copy::<veclib::Vector2<u8>, [u8; size_of::<u8>() * 2]>(self.vertices.uvs.get(idx)?));
                    }
                    if !self.vertices.colors.is_empty() { 
                        big_vector.extend_from_slice(&std::mem::transmute_copy::<veclib::Vector3<u8>, [u8; size_of::<u8>() * 3]>(self.vertices.colors.get(idx)?));
                    }
                }

                gl::GenBuffers(1, buffers.as_mut_ptr().add(1));
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[1]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (big_vector.len()) as isize,
                    big_vector.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // The stride increase each time we add an attribute
                let mut current_stride = 0;

                // Vertex attrib array
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, null());
                current_stride += size_of::<f32>() * 3;

                // Vertex normals attribute
                if !self.vertices.normals.is_empty() {
                    gl::EnableVertexAttribArray(1);
                    gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, stride, current_stride as *const c_void);
                    current_stride += size_of::<i8>() * 3;
                } else {
                    gl::VertexAttrib4Nbv(1, [127, 127, 127, 0_i8].as_ptr());
                }

                if !self.vertices.tangents.is_empty() {
                    // Tangent attribute
                    gl::EnableVertexAttribArray(2);
                    gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, stride, current_stride as *const c_void);
                    current_stride += size_of::<i8>() * 4;
                } else {
                    gl::VertexAttrib4Nbv(2, [0, 0, 0, 127_i8].as_ptr());
                }

                if !self.vertices.uvs.is_empty() {
                    // UV attribute
                    gl::EnableVertexAttribArray(3);
                    gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, stride, current_stride as *const c_void);
                    current_stride += size_of::<u8>() * 2;
                } else {
                    gl::VertexAttrib4Nub(3, 255, 255, 0, 0);
                }

                if !self.vertices.colors.is_empty() {
                    // Vertex colors attribute
                    gl::EnableVertexAttribArray(4);
                    gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, stride, current_stride as *const c_void);
                    //current_stride += size_of::<u8>() * 3;
                } else {
                    gl::VertexAttrib4Nub(4, 255, 255, 255, 0);
                }
            } else {
                gl::GenBuffers(5, buffers.as_mut_ptr().add(1));
                // Normal, fallback
                // Create the vertex buffer and populate it
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[1]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.positions.len() * size_of::<f32>() * 3) as isize,
                    self.vertices.positions.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

                // Vertex attrib array
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

                // Vertex normals attribute
                if !self.vertices.normals.is_empty() {
                    // Vertex normals buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[2]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.vertices.normals.len() * size_of::<i8>() * 3) as isize,
                        self.vertices.normals.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    gl::EnableVertexAttribArray(1);
                    gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, 0, null());
                } else {
                    gl::VertexAttrib4Nbv(1, [127, 127, 127, 0_i8].as_ptr());
                }

                if !self.vertices.tangents.is_empty() {
                    // And it's brother, the tangent buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[3]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.vertices.tangents.len() * size_of::<i8>() * 4) as isize,
                        self.vertices.tangents.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    // Tangent attribute
                    gl::EnableVertexAttribArray(2);
                    gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, 0, null());
                } else {
                    gl::VertexAttrib4Nbv(2, [0, 0, 0, 127_i8].as_ptr());
                }

                if !self.vertices.uvs.is_empty() {
                    // The texture coordinates buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[4]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.vertices.uvs.len() * size_of::<u8>() * 2) as isize,
                        self.vertices.uvs.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    // UV attribute
                    gl::EnableVertexAttribArray(3);
                    gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                } else {
                    gl::VertexAttrib4Nub(3, 255, 255, 0, 0);
                }

                if !self.vertices.colors.is_empty() {
                    // Vertex colors buffer
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[5]);
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (self.vertices.colors.len() * size_of::<u8>() * 3) as isize,
                        self.vertices.colors.as_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );

                    // Vertex colors attribute
                    gl::EnableVertexAttribArray(4);
                    gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                } else {
                    gl::VertexAttrib4Nub(4, 255, 255, 255, 0);
                }
            }

            // Unbind
            self.buffers = buffers;
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Add the model
        pipeline.models.insert(id, self);
        Some(())
    }
    // Remove the model from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let model = pipeline.models.remove(id)?;
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
    // Create a vertex builder
    pub fn vertex_builder(&mut self) -> VertexBuilder {
        VertexBuilder { vertices: &mut self.vertices }
    }
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
        let max_triangle_index: u32 = self.vertices.positions.len() as u32;
        self.triangles.extend(other.triangles.into_iter().map(|mut x| {
            x += max_triangle_index;
            x
        }));
        self.vertices.positions.extend(other.vertices.positions.into_iter());
        self.vertices.normals.extend(other.vertices.normals.into_iter());
        self.vertices.uvs.extend(other.vertices.uvs.into_iter());
        self.vertices.colors.extend(other.vertices.colors.into_iter());
        self.vertices.tangents.extend(other.vertices.tangents.into_iter());
        self
    }
    // Procedurally generate the normals for this model
    pub fn with_generated_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles

        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; self.vertices.positions.len()];
        for i in 0..(self.triangles.len() / 3) {
            // Calculate the face normal
            let (i1, i2, i3) = (self.triangles[i * 3], self.triangles[i * 3 + 1], self.triangles[i * 3 + 2]);
            // Get the actual vertices
            let a = self.vertices.positions.get(i1 as usize).unwrap();
            let b = self.vertices.positions.get(i2 as usize).unwrap();
            let c = self.vertices.positions.get(i3 as usize).unwrap();

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
        self.vertices.normals = vertex_normals.into_iter().map(|x| (x * 127.0).normalized().into()).collect::<Vec<_>>();
    }
}
