use super::{attributes::AttributeSet, GeometryBuilder};
use crate::{
    buffer::{Buffer, BufferMode, ElementBuffer},
    canvas::rasterizer::ToRasterBuffers,
    context::Context,
};

// Specified what attributes are enabled in a vertex set
bitflags::bitflags! {
    pub struct VertexLayout: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 2;
        const TANGENTS = 1 << 3;
        const COLORS = 1 << 4;
        const TEX_COORD_0 = 1 << 5;
    }
}

impl Default for VertexLayout {
    fn default() -> Self {
        Self::empty()
    }
}

// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {
    // The vertex attribute buffers
    attributes: AttributeSet,

    // The index buffer (PS: Supports only triangles rn)
    indices: ElementBuffer<u32>,
}

impl SubMesh {
    // Construct a submesh using a geometry builder
    // This creates a new submesh with attribute layout defined by the builder itself
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    // PS: This doesn't check if the builder contains different length-vectors
    pub(super) unsafe fn new_unchecked(ctx: &mut Context, builder: GeometryBuilder) -> Self {
        Self {
            attributes: AttributeSet::new(ctx, BufferMode::Static, &builder),
            indices: Buffer::new(ctx, BufferMode::Static, builder.get_indices()).unwrap(),
        }
    }

    // Get the current submesh's layout
    pub fn layout(&self) -> VertexLayout {
        self.attributes.layout()
    }

    // Get the underlying attribute set immutably
    pub fn attributes(&self) -> &AttributeSet {
        &self.attributes
    }

    // Get the underlying attribute set mutably
    pub fn attributes_mut(&mut self) -> &mut AttributeSet {
        &mut self.attributes
    }

    // Get the underlying index buffer immutably
    pub fn indices(&self) -> &ElementBuffer<u32> {
        &self.indices
    }

    // Get the underlying index buffer mutably
    pub fn indices_mut(&mut self) -> &mut ElementBuffer<u32> {
        &mut self.indices
    }
}

impl ToRasterBuffers for SubMesh {
    fn vao(&self) -> &AttributeSet {
        self.attributes()
    }

    fn ebo(&self) -> &ElementBuffer<u32> {
        self.indices()
    }
}
