use std::ptr::null;

use crate::{
    advanced::storages::TypedStorage,
    object::Object,
    utils::{AccessType, UpdateFrequency, UsageType},
};

use super::{GeometryBuilder, Indices, MeshBuffers, Vertices};
use arrayvec::ArrayVec;
use assets::Asset;
use getset::{CopyGetters, Getters, Setters};
use gl::types::GLuint;
use obj::TexturedVertex;

// A simple mesh that holds vertex, normal, and color data
#[derive(Default, Getters, CopyGetters, Setters)]
pub struct Mesh {
    // Main IDs
    #[getset(get_copy = "pub(crate)")]
    vao: GLuint,

    // Buffers
    #[getset(get = "pub", get_mut = "pub")]
    buffers: Option<MeshBuffers>,

    /*
    pub element_buffer_object: u32,

    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,

    pub color_buf: u32,
    pub uv_buf: u32,
    */
    // Store the vertices
    #[getset(get = "pub", set = "pub(super)")]
    vertices: Vertices,

    // And indices
    #[getset(get = "pub", set = "pub(super)")]
    indices: Indices,
}

impl Asset for Mesh {
    fn deserialize(self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let parsed_obj = obj::load_obj::<TexturedVertex, &[u8], u32>(bytes).unwrap();
        // Generate the tangents
        // Create the actual Mesh now
        let mut builder = GeometryBuilder::default();
        let vertices = &mut builder.vertices;
        for vertex in parsed_obj.vertices {
            vertices.position(vek::Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2]));
            vertices.normal(vek::Vec3::new(
                (vertex.normal[0] * 127.0) as i8,
                (vertex.normal[1] * 127.0) as i8,
                (vertex.normal[2] * 127.0) as i8,
            ));
            vertices.uv(vek::Vec2::new((vertex.texture[0] * 255.0) as u8, (vertex.texture[1] * 255.0) as u8));
        }
        builder.indices.indices = parsed_obj.indices;
        Some(builder.build())
    }
}

impl Object for Mesh {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        // Create the OpenGL mesh (even if it is empty)
        unsafe {
            // Create the VAO
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);

            // Usage
            let usage = UsageType {
                access: AccessType::ClientToServer,
                frequency: UpdateFrequency::WriteOnceReadMany,
                dynamic: false,
            };

            // All the buffers
            let mut buffers = ArrayVec::<GLuint, 6>::default();

            // Create the EBO
            let indices = TypedStorage::<u32>::new(self.indices().len(), self.indices().len(), self.indices().as_ptr(), gl::ELEMENT_ARRAY_BUFFER, usage);
            buffers.push(indices.raw().buffer());

            // Positions
            let positions = TypedStorage::<vek::Vec3<f32>>::new(self.vertices().len(), self.vertices().len(), self.vertices.positions.as_ptr(), gl::ARRAY_BUFFER, usage);
            buffers.push(positions.raw().buffer());

            // Vertex attrib array
            gl::BindBuffer(gl::ARRAY_BUFFER, positions.raw().buffer());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, null());

            // Vertex normals attribute
            let normals = if !self.vertices.normals.is_empty() {
                // Vertex normals buffer
                let normals = TypedStorage::<vek::Vec3<i8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.normals.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(normals.raw().buffer());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, 0, null());
                Some(normals)
            } else {
                gl::VertexAttrib4Nbv(1, [127, 127, 127, 0_i8].as_ptr());
                None
            };

            let tangents = if !self.vertices.tangents.is_empty() {
                // Vertex tangents buffer
                let tangents = TypedStorage::<vek::Vec4<i8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.tangents.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(tangents.raw().buffer());
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, 0, null());
                Some(tangents)
            } else {
                gl::VertexAttrib4Nbv(2, [0, 0, 0, 127_i8].as_ptr());
                None
            };

            let uvs = if !self.vertices.uvs.is_empty() {
                // Vertex texture coordinates buffer
                let uvs = TypedStorage::<vek::Vec2<u8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.uvs.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(uvs.raw().buffer());
                gl::EnableVertexAttribArray(3);
                gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                Some(uvs)
            } else {
                gl::VertexAttrib4Nub(3, 255, 255, 0, 0);
                None
            };

            let colors = if !self.vertices.colors.is_empty() {
                // Vertex colors buffer
                let colors = TypedStorage::<vek::Rgb<u8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.colors.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(colors.raw().buffer());
                gl::EnableVertexAttribArray(4);
                gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                Some(colors)
            } else {
                gl::VertexAttrib4Nub(4, 255, 255, 255, 0);
                None
            };
            // Unbind
            self.buffers = Some(MeshBuffers {
                inner: buffers,
                indices,
                positions,
                normals,
                tangents,
                colors,
                uvs,
            });
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    fn disposed(self) {
        unsafe {
            // Delete the vertex array
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Mesh {
    // Create a new mesh using raw vertices and indices
    pub fn new(vertices: Vertices, indices: Indices) -> Self {
        Self {
            vertices,
            indices,
            ..Default::default()
        }
    }
    // Créer un nouveau mesh en combinant deux meshs qui existent déja.
    pub fn combine(mut self, other: Mesh) -> Mesh {
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
    pub fn generate_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles
        let mut vertex_normals: Vec<vek::Vec3<f32>> = vec![vek::Vec3::zero(); self.vertices.positions.len()];
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
            let cross = vek::Vec3::<f32>::cross(d1, d2).normalized();

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
        self.vertices.normals = vertex_normals.into_iter().map(|x| (x * 127.0).normalized().as_()).collect::<Vec<_>>();
    }
}
