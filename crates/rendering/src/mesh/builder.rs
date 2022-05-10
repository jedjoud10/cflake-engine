use super::{vertex::*, NamedAttribute, SubMesh, VertexLayout};
use crate::context::Context;

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
        self.layout.insert(U::LAYOUT);
    }

    // Set the indices alone lul
    pub fn set_indices(&mut self, vec: Vec<u32>) {
        self.indices = vec;
    }

    // Get the specific vertex layout that this geometry builder will use to build it's submesh
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Check if the vectors are valid (AKA they have the same length)
    pub fn valid(&self) -> bool {
        let first = self.positions.len();
        let arr = [self.normals.len(), self.tangents.len(), self.colors.len(), self.tex_coord_0.len()];
        arr.into_iter().all(|len| len == first)
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
