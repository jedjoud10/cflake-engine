use std::ptr::null;

use crate::{
    advanced::storages::TypedStorage,
    object::ObjectSealed,
    utils::{AccessType, UpdateFrequency, UsageType},
};

use super::{GeometryBuilder, Indices, MeshBuffers, MeshFlags, Vertices};
use arrayvec::ArrayVec;
use assets::Asset;
use getset::{CopyGetters, Getters, Setters};
use gl::types::GLuint;
use math::bounds::aabb::AABB;
use obj::TexturedVertex;

// A simple mesh that holds vertex, normal, and color data
#[derive(Getters, CopyGetters, Setters)]
pub struct Mesh {
    // Main IDs
    #[getset(get_copy = "pub(crate)")]
    vao: GLuint,

    // Buffers
    #[getset(get = "pub", get_mut = "pub(crate)")]
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

    // Mesh flags telling us what vertex attributes are suported and shit
    #[getset(get = "pub")]
    flags: MeshFlags,

    // Mesh limits, can be used for culling
    #[getset(get = "pub")]
    bounds: AABB,
}

impl Asset for Mesh {
    type OptArgs = ();
    fn deserialize(_meta: &assets::metadata::AssetMetadata, bytes: &[u8], _input: Self::OptArgs) -> Option<Self>
    where
        Self: Sized,
    {
        // Parse the OBJ mesh into an engine mesh
        let parsed_obj = obj::load_obj::<TexturedVertex, &[u8], u32>(bytes).unwrap();
        let mut builder = GeometryBuilder::default();

        // Load each vertex SoA style
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

        // Also load the triangles
        builder.indices.indices = parsed_obj.indices;

        // Compute the tangents automatically for imported meshes
        let mesh = builder.build().generate_tangents();
        Some(mesh)
    }
}

impl ObjectSealed for Mesh {
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
            let normals = if self.flags.contains(MeshFlags::NORMALS_SUPPORTED) {
                // Vertex normals buffer
                let normals = TypedStorage::<vek::Vec3<i8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.normals.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(normals.raw().buffer());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 3, gl::BYTE, gl::TRUE, 0, null());
                Some(normals)
            } else {
                // Default normal is mid
                gl::VertexAttrib4Nbv(1, [127, 127, 127, 0_i8].as_ptr());
                None
            };

            // Vertex tangents attribute
            let tangents = if self.flags.contains(MeshFlags::TANGENTS_SUPPORTED) {
                // Vertex tangents buffer
                let tangents = TypedStorage::<vek::Vec4<i8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.tangents.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(tangents.raw().buffer());
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 4, gl::BYTE, gl::TRUE, 0, null());
                Some(tangents)
            } else {
                // Default tangent is uhhhh, fard
                gl::VertexAttrib4Nbv(2, [0, 0, 0, 127_i8].as_ptr());
                None
            };

            // Vertex texture coordinates attribute
            let uvs = if self.flags.contains(MeshFlags::UVS_SUPPORTED) {
                // Vertex texture coordinates buffer
                let uvs = TypedStorage::<vek::Vec2<u8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.uvs.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(uvs.raw().buffer());
                gl::EnableVertexAttribArray(3);
                gl::VertexAttribPointer(3, 2, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                Some(uvs)
            } else {
                // Default UV is one
                gl::VertexAttrib4Nub(3, 255, 255, 0, 0);
                None
            };

            // Vertex colors attribute
            let colors = if self.flags.contains(MeshFlags::VERTEX_COLORS_SUPPORTED) {
                // Vertex colors buffer
                let colors = TypedStorage::<vek::Rgb<u8>>::new(self.vertices().len(), self.vertices().len(), self.vertices.colors.as_ptr(), gl::ARRAY_BUFFER, usage);
                buffers.push(colors.raw().buffer());
                gl::EnableVertexAttribArray(4);
                gl::VertexAttribPointer(4, 3, gl::UNSIGNED_BYTE, gl::TRUE, 0, null());
                Some(colors)
            } else {
                // Default color is white
                gl::VertexAttrib4Nub(4, 255, 255, 255, 0);
                None
            };

            // Create le buffer from the attributes
            self.buffers = Some(MeshBuffers {
                inner: buffers,
                indices,
                positions,
                normals,
                tangents,
                colors,
                uvs,
            });

            // Rinse and repeat
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // Don't forget to calculate the AABB while we're at it
            self.update_bounds();
        }
    }

    fn disposed(self) {
        unsafe {
            // Lul mem leak
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Mesh {
    // Create a new mesh using raw vertices and indices
    pub fn new(vertices: Vertices, indices: Indices) -> Self {
        Self {
            flags: MeshFlags::get(&vertices),
            vertices,
            indices,
            vao: 0,
            buffers: None,
            bounds: AABB::default(),
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
    // Flip the model inside out
    pub fn flip_triangles(mut self) -> Self {
        for base in self.indices.chunks_mut(3) {
            base.swap(0, 2);
        }
        self
    }
    // Recalculate the mesh bounds
    pub fn update_bounds(&mut self) {
        // Keep track of the min/max positions
        let mut max = vek::Vec3::broadcast(f32::MIN);
        let mut min = vek::Vec3::broadcast(f32::MAX);

        for vert in &self.vertices.positions {
            // Iterate for each element in the current point and update the min/max values
            vert.iter().enumerate().for_each(|(i, e)| {
                max[i] = e.max(max[i]);
                min[i] = e.min(min[i]);
            })
        }

        // Update bounds
        self.bounds.min = min;
        self.bounds.max = max;
    }
    // Procedurally generate the normals for this mesh
    pub fn generate_normals(mut self) -> Self {
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
        self.flags.insert(MeshFlags::NORMALS_SUPPORTED);
        self
    }
    // Procedurally generate the tangents for this mesh (given the normals and UVs)
    pub fn generate_tangents(mut self) -> Self {
        // Check if we can even generate the tangents, and return early if we cannot
        if !self.flags.contains(MeshFlags::NORMALS_SUPPORTED) || !self.flags.contains(MeshFlags::UVS_SUPPORTED) {
            return self;
        }

        // Local struct just cause I don't want to implement mikktspace::Geometry for Mesh
        struct TangentGenerator<'a> {
            // Values read from the mesh
            positions: &'a [vek::Vec3<f32>],
            indices: &'a [u32],
            normals: &'a [vek::Vec3<i8>],
            uvs: &'a [vek::Vec2<u8>],

            // Tangents that we will write to (array is already pre-allocated, so we can just write directly)
            tangents: &'a mut [vek::Vec4<i8>],
        }

        // I love external libraries
        impl<'a> mikktspace::Geometry for TangentGenerator<'a> {
            fn num_faces(&self) -> usize {
                self.indices.len() / 3
            }

            // All the models must be triangulated, so we are gud
            fn num_vertices_of_face(&self, _face: usize) -> usize {
                3
            }

            // Read position using index magic
            fn position(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.positions[i].into_array()
            }

            // Read normal using index magic
            fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
                let i = self.indices[face * 3 + vert] as usize;
                self.normals[i].map(|x| x as f32 / 127.0).into_array()
            }

            // Read texture coordinate using index magic
            fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
                let i = self.indices[face * 3 + vert] as usize;
                self.uvs[i].map(|x| x as f32 / 255.0).into_array()
            }

            // Write a tangent internally
            fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
                let i = self.indices[face * 3 + vert] as usize;
                self.tangents[i] = vek::Vec4::<f32>::from_slice(&tangent).map(|x| (x * 127.0) as i8);
            }
        }

        // Pre-allocate the tangents and create the generator
        let mut tangents = vec![vek::Vec4::<i8>::zero(); self.vertices.len()];
        let mut gen = TangentGenerator {
            positions: &self.vertices.positions,
            normals: &self.vertices.normals,
            indices: &self.indices,
            uvs: &self.vertices.uvs,
            tangents: &mut tangents,
        };

        // Le panic
        assert!(mikktspace::generate_tangents(&mut gen), "Something went wrong when generating tangents!");

        // Update our tangents
        self.vertices.tangents = tangents;
        self.flags.insert(MeshFlags::TANGENTS_SUPPORTED);
        self
    }
}
