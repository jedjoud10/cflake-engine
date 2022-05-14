use super::{vertex::*, NamedAttribute, SubMesh, VertexLayout, VertexAssembly, TriangleAssembly};
use crate::{
    context::Context,
    mesh::{Color, Normal, Tangent, TexCoord0},
};

// Procedural geometry builder that will help us generate submeshes
// This however, can be made in other threads and then sent to the main thread
#[derive(Default)]
pub struct GeometryBuilder {
    // Vertices and their attributes
    vertices: VertexAssembly,

    // Indices stored as triangles
    triangles: TriangleAssembly,
}

impl GeometryBuilder {
    // Set a single unique vertex attribute
    pub fn set_attrib<U: NamedAttribute>(&mut self, vec: Vec<U::Out>) {
        self.vertices.insert::<U>(vec);
    }

    // Get a vertex attribute immutably
    pub fn get_attrib<U: NamedAttribute>(&self) -> Option<&Vec<U::Out>> {
        self.vertices.get::<U>()
    }

    // Get an attribute vector mutably
    pub fn get_attrib_mut<U: NamedAttribute>(&mut self) -> Option<&mut Vec<U::Out>> {
        self.vertices.get_mut::<U>()
    }

    // Get the vertex layout that we have created
    pub fn layout(&self) -> VertexLayout {
        self.vertices.layout()
    }

    // Set the indices (triangles)
    pub fn set_tris(&mut self, assembly: TriangleAssembly) {
        self.triangles = assembly;
    }

    // Get the indices (triangles) immutably 
    pub fn get_tris(&self) -> &TriangleAssembly {
        &self.triangles
    }

    // Get the indices (triangles) mutably
    pub fn get_tris_mut(&mut self) -> &mut TriangleAssembly {
        &mut self.triangles
    }

    // Check if the builder can be used to generate a submesh
    pub fn valid(&self) -> bool {
        self.vertices.len().is_some()
    }

    // Build the final submesh without checking for validity or anything
    pub unsafe fn build_unchecked(self, ctx: &mut Context) -> SubMesh {
        SubMesh::new_unchecked(ctx, self)
    }

    // Build the final submesh using a specific context, and make sure the vecs are valid
    pub fn build(self, ctx: &mut Context) -> Option<SubMesh> {
        self.valid().then(|| unsafe { SubMesh::new_unchecked(ctx, self) })
    }
}
