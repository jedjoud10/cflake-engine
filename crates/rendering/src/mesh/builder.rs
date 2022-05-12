use super::{vertex::*, NamedAttribute, SubMesh, VertexLayout};
use crate::{context::Context, mesh::{Normal, TexCoord0, Tangent, Color}};

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

impl Default for GeometryBuilder {
    fn default() -> Self {
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
} 

impl GeometryBuilder {
    // Set each type of attribute vector using trait magic
    pub fn set<U: NamedAttribute>(&mut self, vec: Vec<U::Out>) {
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

    // Check if the vectors are valid
    pub fn valid(&self) -> bool {
        // Check if a vertex layout type is present within our current vertex layout, and return a usize that represents it (1 for true, 0 for false)
        fn valid<A: NamedAttribute>(builder: &GeometryBuilder, count: usize, vec: &Vec<A::Out>) -> bool {
            let factor = usize::from(builder.layout.contains(A::LAYOUT));
            (count - factor * vec.len()) == 0
        }  

        // If we have no position vertices, then wtf we doing bruv?
        let length = if self.positions.is_empty() {
            return true;
        } else { self.positions.len() };

        // Check if each vector is valid
        let valids = [
            valid::<Normal>(&self, length, &self.normals),
            valid::<Tangent>(&self, length, &self.tangents),
            valid::<Color>(&self, length, &self.colors),
            valid::<TexCoord0>(&self, length, &self.tex_coord_0)
        ];

        // They must ALL be valid
        valids.into_iter().all(|a| a)
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
