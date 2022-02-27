use super::Vertices;
use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
    utils::UpdateFrequency,
};
use assets::Asset;
use gl::types::GLuint;
use obj::TexturedVertex;
use std::{ffi::c_void, mem::size_of, ptr::null};
use veclib::{vec2, vec3};

// A simple mesh that holds vertex, normal, and color data
pub struct Mesh {
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

    // Update frequence
    pub update_frequency: UpdateFrequency,

    // Triangles
    pub indices: Vec<u32>,

    // Keep track of the number of vertices and triangles since we might clear the CPU buffers
    pub vert_count: usize,
    pub tris_count: usize,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertex_array_object: Default::default(),
            buffers: Default::default(),
            vertices: Default::default(),
            update_frequency: UpdateFrequency::Static,
            indices: Default::default(),
            vert_count: Default::default(),
            tris_count: Default::default(),
        }
    }
}

impl PipelineObject for Mesh {
    // Reserve an ID for this mesh
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.meshes.gen_id()))
    }
    // Send this mesh to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Mesh(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Mesh(Deconstruct::<Self>(id))
    }
    // Add the mesh to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the mesh
        if self.vertices.len() > 0 {
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
                    (self.indices.len() * size_of::<u32>()) as isize,
                    self.indices.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );

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

                // Clear the CPU buffers if we want to
                self.tris_count = self.indices.len() / 3;
                self.vert_count = self.vertices.len();
                if let UpdateFrequency::Static = self.update_frequency {
                    //self.vertices.reset();
                    //self.indices.drain(..);
                }

                // Unbind
                self.buffers = buffers;
                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
        // Add the mesh
        pipeline.meshes.insert(id, self);
        Some(())
    }
    // Remove the mesh from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let mesh = pipeline.meshes.remove(id)?;
        // Dispose of the OpenGL buffers
        unsafe {
            if mesh.vertices.len() > 0 {
                // Delete the VBOs
                gl::DeleteBuffers(mesh.buffers.len() as i32, mesh.buffers.as_ptr());

                // Delete the vertex array
                gl::DeleteVertexArrays(1, &mesh.vertex_array_object);
            }
        }
        Some(mesh)
    }
}

impl Mesh {
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.indices.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.indices.swap(i, i + 2);
        }
    }
    // Combine a mesh with this one, and return the new mesh
    pub fn combine(mut self, other: Self) -> Self {
        let max_triangle_index: u32 = self.vertices.positions.len() as u32;
        self.indices.extend(other.indices.into_iter().map(|mut x| {
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
    // Procedurally generate the normals for this mesh
    pub fn with_generated_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles

        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; self.vertices.positions.len()];
        for i in 0..(self.indices.len() / 3) {
            // Calculate the face normal
            let (i1, i2, i3) = (self.indices[i * 3], self.indices[i * 3 + 1], self.indices[i * 3 + 2]);
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

impl Asset for Mesh {
    // Load a mesh from an asset file
    fn deserialize(mut self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let parsed_obj = obj::load_obj::<TexturedVertex, &[u8], u32>(bytes).unwrap();
        // Generate the tangents
        // Create the actual Mesh now
        for vertex in parsed_obj.vertices {
            self.vertices
                .add()
                .with_position(vec3(vertex.position[0], vertex.position[1], vertex.position[2]))
                .with_normal(vec3((vertex.normal[0] * 127.0) as i8, (vertex.normal[1] * 127.0) as i8, (vertex.normal[2] * 127.0) as i8))
                .with_uv(vec2((vertex.texture[0] * 255.0) as u8, (vertex.texture[1] * 255.0) as u8));
        }
        self.indices = parsed_obj.indices;
        Some(self)
    }
}
