use super::{attributes::Attribute, IndexAssembly, SubMesh, VertexAssembly};
use crate::{buffer::BufferMode, context::Context};
use assets::Asset;
use obj::TexturedVertex;

// Procedural geometry builder that will help us generate submeshes
// This however, can be made in other threads and then sent to the main thread
pub struct GeometryBuilder {
    // Vertices and their attributes
    vertices: VertexAssembly,

    // Indices stored as triangles
    indices: Vec<u32>,
}

impl Default for GeometryBuilder {
    fn default() -> Self {
        Self::empty()
    }
}

impl GeometryBuilder {
    // Create a new empty procedular geometry builder
    // It contains no attributes or indices, just an empty one
    pub fn empty() -> Self {
        Self {
            vertices: VertexAssembly::empty(),
            indices: Vec::new(),
        }
    }

    // Set a single unique vertex attribute
    pub fn with_attribute_vec<U: Attribute>(mut self, vec: Vec<U::Out>) -> Self {
        self.vertices.insert::<U>(vec);
        self
    }

    // Set the indices
    pub fn with_indices(mut self, assembly: IndexAssembly) -> Self {
        self.indices = assembly;
        self
    }

    // Check if the builder can be used to generate a submesh
    pub fn valid(&self) -> bool {
        // We must have at least 1 vertex and at least 1 triangle and the underlying vertex assembly is valid
        let tri = self.indices.len() % 3 == 0;
        let vert = self.vertices.len().map(|len| len > 1).unwrap_or_default();
        vert && tri
    }

    // Build the final submesh without checking for validity or anything
    pub unsafe fn build_unchecked(self, ctx: &mut Context, mode: BufferMode) -> SubMesh {
        SubMesh::new_unchecked(ctx, self.vertices, self.indices, mode)
    }

    // Build the final submesh, and make sure the attribute vectors are valid
    pub fn build(self, ctx: &mut Context, mode: BufferMode) -> Option<SubMesh> {
        self.valid()
            .then(|| unsafe { SubMesh::new_unchecked(ctx, self.vertices, self.indices, mode) })
    }
}

impl Asset<'static> for GeometryBuilder {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(data: assets::Data, _ctx: Self::Args) -> Self {
        // Parse the OBJ mesh into the different vertex attributes and the indices
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();
        let capacity = parsed.vertices.len();

        // Create all the buffers at once
        let mut positions = Vec::with_capacity(capacity);
        let mut normals = Vec::with_capacity(capacity);
        let mut tex_coords_0 = Vec::with_capacity(capacity);
        let indices = parsed.indices;

        // Fill each buffer now
        use super::attributes::named::*;
        use vek::{Vec2, Vec3};
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position));
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }

        // We will now calculate the tangents for each vertex using some math magic
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
                self.tangents[i] =
                    vek::Vec4::<f32>::from_slice(&tangent).map(|x| (x * 127.0) as i8);
            }
        }

        // Pre-allocate the tangents and create the mikktspace generator
        let mut tangents = vec![vek::Vec4::<i8>::zero(); capacity];
        let mut gen = TangentGenerator {
            positions: &positions,
            normals: &normals,
            indices: &indices,
            uvs: &tex_coords_0,
            tangents: &mut tangents,
        };

        // Generate the procedural tangents
        assert!(mikktspace::generate_tangents(&mut gen));

        // Set the very sussy bakas (POV: You are slowly going insane)

        GeometryBuilder::empty()
            .with_attribute_vec::<Position>(positions)
            .with_attribute_vec::<Normal>(normals)
            .with_attribute_vec::<Tangent>(tangents)
            .with_attribute_vec::<TexCoord0>(tex_coords_0)
            .with_indices(indices)
    }
}
