use crate::{
    context::Context,
};

use super::{VertexAssembly, attributes::NamedAttribute, VertexLayout, IndexAssembly, SubMesh};

// Procedural geometry builder that will help us generate submeshes
// This however, can be made in other threads and then sent to the main thread
#[derive(Default)]
pub struct GeometryBuilder {
    // Vertices and their attributes
    vertices: VertexAssembly,

    // Indices stored as triangles
    indices: Vec<u32>,
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

    // Set the indices
    pub fn set_indices(&mut self, assembly: IndexAssembly) {
        self.indices = assembly;
    }

    // Get the indices immutably 
    pub fn get_indices(&self) -> &IndexAssembly {
        &self.indices
    }

    // Get the indices mutably
    pub fn get_indices_mut(&mut self) -> &mut IndexAssembly {
        &mut self.indices
    }

    // Check if the builder can be used to generate a submesh
    pub fn valid(&self) -> bool {
        self.vertices.len().is_some() && self.indices.len() % 3 == 0
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
