use crate::context::Context;

use super::{vertex::*, NamedAttribute, SubMesh, VertexLayout};


// Procedural geometry builder that will help us generate submeshes 
// This however, can be made in other threads and then sent to the main thread
pub struct GeometryBuilder {
    // Rust vectors of vertex attributes 
    pub(super) positions: Vec<VePos>,
    pub(super) normals: Vec<VeNormal>,
    pub(super) tangents: Vec<VeTangent>,
    pub(super) colors: Vec<VeColor>,
    pub(super) tex_coord_0: Vec<VeTexCoord0>,

    // Index vector
    pub(super) indices: Vec<u32>,

    // Other
    layout: VertexLayout,
}

impl GeometryBuilder {
    // Create a new geometry builder
    pub fn new() -> Self {
        Self {
            positions: Default::default(),
            normals: Default::default(),
            tangents: Default::default(),
            colors: Default::default(),
            tex_coord_0: Default::default(),
            indices: Default::default(),
            layout: VertexLayout::empty(),
        }
    }

    // Set each type of attribute vector using trait magic
    pub fn insert<U: NamedAttribute>(&mut self, vec: Vec<U::Out>) {
        U::insert(self, vec);
        self.layout.insert(U::LAYOUT_ID);
    }

    // Set the indices alone lul
    pub fn set_indices(&mut self, vec: Vec<u32>) {
        self.indices = vec;
    }

    // Get the specific vertex layout that this geometry builder will use to build it's submesh
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Build the final submesh using a specific context
    pub fn build(ctx: &mut Context) -> SubMesh {
        todo!()
    }
}