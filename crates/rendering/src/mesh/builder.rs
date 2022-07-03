use super::{attributes::Attribute, IndexAssembly, SubMesh, VertexAssembly, VertexLayout};
use crate::context::Context;
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
    pub unsafe fn build_unchecked(self, ctx: &mut Context) -> SubMesh {
        SubMesh::new_unchecked(ctx, self.vertices, self.indices)
    }

    // Build the final submesh, and make sure the attribute vectors are valid
    pub fn build(self, ctx: &mut Context) -> Option<SubMesh> {
        self.valid()
            .then(|| unsafe { SubMesh::new_unchecked(ctx, self.vertices, self.indices) })
    }
}

impl Asset<'static> for GeometryBuilder {
    type Args = ();

    fn extensions() -> &'static [&'static str] {
        &["obj"]
    }

    fn deserialize(data: assets::Data, _ctx: Self::Args) -> Self {
        // Parse the OBJ mesh into an geoemtry builder
        let parsed = obj::load_obj::<TexturedVertex, &[u8], u32>(data.bytes()).unwrap();
        let capacity = parsed.vertices.len();

        // Create all the buffers at once
        let mut positions = Vec::with_capacity(capacity);
        let mut normals = Vec::with_capacity(capacity);
        let mut tex_coords_0 = Vec::with_capacity(capacity);

        // Fill each buffer now
        use super::attributes::named::*;
        use vek::{Vec2, Vec3};
        for vertex in parsed.vertices {
            positions.push(Vec3::from_slice(&vertex.position));
            normals.push(Vec3::from_slice(&vertex.normal).map(|f| (f * 127.0) as i8));
            tex_coords_0.push(Vec2::from_slice(&vertex.texture).map(|f| (f * 255.0) as u8));
        }

        // Set the very sussy bakas (POV: You are slowly going insane)
        let builder = GeometryBuilder::empty()
            .with_attribute_vec::<Position>(positions)
            .with_attribute_vec::<Normal>(normals)
            .with_attribute_vec::<TexCoord0>(tex_coords_0)
            .with_indices(parsed.indices);

        builder
    }
}
