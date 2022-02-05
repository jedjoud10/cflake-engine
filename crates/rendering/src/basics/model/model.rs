use super::{CustomVertexDataBuffer, StoredCustomVertexDataBuffer};
use crate::{
    object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject},
    pipeline::Pipeline,
    utils::DataType,
};
use std::{ffi::c_void, fmt::Debug, mem::size_of, ptr::null};

// Some OpenGL data for a model
#[derive(Default, Debug)]
pub struct ModelBuffers {
    // The OpenGL data
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,
    pub uv_buf: u32,
    pub color_buf: u32,
    // Some custom vertex data if we want
    pub custom_vertex_data: u32,

    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub triangle_count: usize,
}

// A simple model that holds vertex, normal, and color data
#[derive(Default)]
pub struct Model {
    // Per vertex data
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub(crate) custom: Option<StoredCustomVertexDataBuffer>,

    // Triangles
    pub triangles: Vec<u32>,
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
    fn add(self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Add the model
        let mut buffers = ModelBuffers::default();
        buffers.triangle_count = self.triangles.len();
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut buffers.vertex_array_object);
            gl::BindVertexArray(buffers.vertex_array_object);

            // Create the EBO
            gl::GenBuffers(1, &mut buffers.element_buffer_object);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers.element_buffer_object);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.triangles.len() * size_of::<u32>()) as isize,
                self.triangles.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the vertex buffer and populate it
            gl::GenBuffers(1, &mut buffers.vertex_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.vertex_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * size_of::<f32>() * 3) as isize,
                self.vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Create the normals buffer
            gl::GenBuffers(1, &mut buffers.normal_buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.normal_buf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.normals.len() * size_of::<f32>() * 3) as isize,
                self.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            if !self.tangents.is_empty() {
                // And it's brother, the tangent buffer
                gl::GenBuffers(1, &mut buffers.tangent_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.tangent_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.tangents.len() * size_of::<f32>() * 4) as isize,
                    self.tangents.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !self.uvs.is_empty() {
                // The texture coordinates buffer
                gl::GenBuffers(1, &mut buffers.uv_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.uv_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.uvs.len() * size_of::<f32>() * 2) as isize,
                    self.uvs.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            if !self.colors.is_empty() {
                // Finally, the vertex colors buffer
                gl::GenBuffers(1, &mut buffers.color_buf);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.color_buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.colors.len() * size_of::<f32>() * 3) as isize,
                    self.colors.as_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            // Add some custom data if we want to
            if self.custom.is_some() {
                // Custom data moment
                gl::GenBuffers(1, &mut buffers.custom_vertex_data);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.custom_vertex_data);
                let stored = self.custom.as_ref().unwrap();
                gl::BufferData(gl::ARRAY_BUFFER, stored.inner.len() as isize, stored.inner.as_ptr() as *const c_void, gl::STATIC_DRAW);
            }

            // Create the vertex attrib arrays
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.vertex_buf);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Normal attribute
            gl::EnableVertexAttribArray(1);
            gl::BindBuffer(gl::ARRAY_BUFFER, buffers.normal_buf);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

            if !self.tangents.is_empty() {
                // Tangent attribute
                gl::EnableVertexAttribArray(2);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.tangent_buf);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !self.uvs.is_empty() {
                // UV attribute
                gl::EnableVertexAttribArray(3);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.uv_buf);
                gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, 0, null());
            }
            if !self.colors.is_empty() {
                // Vertex color attribute
                gl::EnableVertexAttribArray(4);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.color_buf);
                gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, 0, null());
            }
            if self.custom.is_some() {
                // Vertex custom attribute
                let custom = self.custom.as_ref().unwrap();
                gl::EnableVertexAttribArray(5);
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers.custom_vertex_data);
                match custom._type {
                    DataType::F32 => {
                        // Float point
                        gl::VertexAttribPointer(5, custom.components_per_vertex as i32, custom._type.convert(), gl::FALSE, 0, null());
                    }
                    x => {
                        // Integer
                        gl::VertexAttribIPointer(5, custom.components_per_vertex as i32, x.convert(), 0, null());
                    }
                }
            }
            // Unbind
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        // Add the model
        pipeline.models.insert(id.get()?, (self, buffers));
        Some(())
    }
    // Remove the model from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        let (model, mut buffers) = pipeline.models.remove(id.get()?)?;
        // Dispose of the OpenGL buffers
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(1, &mut buffers.vertex_buf);
            gl::DeleteBuffers(1, &mut buffers.normal_buf);
            gl::DeleteBuffers(1, &mut buffers.uv_buf);
            gl::DeleteBuffers(1, &mut buffers.tangent_buf);
            gl::DeleteBuffers(1, &mut buffers.color_buf);
            gl::DeleteBuffers(1, &mut buffers.element_buffer_object);
            gl::DeleteBuffers(1, &mut buffers.custom_vertex_data);

            // Delete the vertex array
            gl::DeleteVertexArrays(1, &mut buffers.vertex_array_object);
        }
        Some(model)
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("vertices", &self.vertices.len())
            .field("normals", &self.normals.len())
            .field("tangents", &self.tangents.len())
            .field("uvs", &self.uvs.len())
            .field("colors", &self.colors.len())
            .field("triangles", &self.triangles.len())
            .finish()
    }
}

impl Model {
    // Create a new model with some certain capacity to hold a number of vertices
    pub fn with_capacity(vertices_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices_count),
            normals: Vec::with_capacity(vertices_count),
            tangents: Vec::with_capacity(vertices_count),
            uvs: Vec::with_capacity(vertices_count),
            colors: Vec::with_capacity(vertices_count),
            custom: None,
            triangles: Vec::with_capacity(vertices_count * 3),
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
    pub fn combine(mut self, other: Self) -> Self {
        // We must have matching custom vertex buffers
        if self.custom.is_some() != other.custom.is_some() {
            panic!()
        };

        let max_triangle_index: u32 = self.vertices.len() as u32;
        // Get the max triangle inde
        let mut final_tris = other.triangles.clone();
        for x in final_tris.iter_mut() {
            *x += max_triangle_index;
        }
        self.triangles.extend(final_tris);
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        if let Some(custom) = other.custom {
            self.custom.as_mut().unwrap().inner.extend(custom.inner.into_iter());
        }
        self
    }
    // Combine a model with this one
    // NOTE: This assumes that the second model uses vertices from the first model
    pub fn combine_smart(mut self, other: Self) -> Self {
        // We must have matching custom vertex buffers
        if self.custom.is_some() != other.custom.is_some() {
            panic!()
        };

        self.triangles.extend(other.triangles.into_iter());
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        self
    }
    // Procedurally generate the normals for this model
    pub fn generate_normals(mut self) {
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
        self.normals = vertex_normals;
    }
    // Add some custom vertex data
    pub fn with_custom<T>(mut self, custom: CustomVertexDataBuffer<T>, _type: DataType) -> Self {
        self.custom = Some(StoredCustomVertexDataBuffer::new(custom, _type));
        self
    }
}
