use super::{
    vao::{standard::StandardAttributeSet},
    GeometryBuilder,
};
use crate::{
    buffer::{Buffer, BufferMode, ElementBuffer},
    context::{Context},
    object::{ToGlName},
};
use assets::Asset;
use math::bounds::aabb::AABB;
use obj::TexturedVertex;
use std::{mem::ManuallyDrop, num::NonZeroU32, ptr::null};

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

// Tells us how we should store the vertex attributes within the submesh
pub trait VertexStorage {}

impl VertexStorage for StandardAttributeSet {

}

// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {
    // The vertex attribute buffers
    attributes: StandardAttributeSet,

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
            attributes: StandardAttributeSet::new(ctx, BufferMode::Static, &builder),
            indices: Buffer::new(ctx, BufferMode::Static, builder.get_indices()).unwrap(),
        }
    }

    // Get the current submesh's layout
    pub fn layout(&self) -> VertexLayout {
        self.attributes.layout()
    }

    // Get the underlying attribute set immutably
    pub fn attributes(&self) -> &StandardAttributeSet {
        &self.attributes
    }

    // Get the underlying attribute set mutably
    pub fn attributes_mut(&mut self) -> &mut StandardAttributeSet {
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

/*
impl<'ctx> Asset<'ctx> for Vec<SubMesh> {
    type Args = &'ctx mut Context;

    fn extensions() -> &'static [&'static str] {
        GeometryBuilder::extensions()
    }

    fn deserialize(bytes: assets::loader::CachedSlice, ctx: Self::Args) -> Self {
        let main = GeometryBuilder::deserialize(bytes, ()).build(ctx).unwrap();
        Self::from_submeshes(ctx, vec![main])
    }
}
*/