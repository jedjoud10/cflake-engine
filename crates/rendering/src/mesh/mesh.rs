use super::{
    attributes::{AttributeSet, NamedAttribute},
    GeometryBuilder,
};
use crate::{
    buffer::{Buffer, BufferMode, ElementBuffer},
    context::{Cached, Context},
    object::ToGlName,
};
use assets::Asset;
use math::bounds::aabb::AABB;
use obj::TexturedVertex;
use std::{mem::ManuallyDrop, num::NonZeroU32};

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
    // The OpenGL VAO name
    vao: NonZeroU32,

    // The vertex attribute buffers
    attributes: AttributeSet,

    // The index buffer (PS: Supports only triangles rn)
    indices: ElementBuffer,
}

impl SubMesh {
    // Construct a submesh using a geometry builder
    // This creates a new submesh with attribute layout defined by the builder itself
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    // PS: This doesn't check if the builder contains different length-vectors
    pub(super) unsafe fn new_unchecked(ctx: &mut Context, builder: GeometryBuilder) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            NonZeroU32::new(name).unwrap()
        };

        Self {
            vao,
            attributes: AttributeSet::new(vao, ctx, BufferMode::Static, &builder),
            indices: Buffer::new(ctx, BufferMode::Static, builder.get_indices()).unwrap(),
        }
    }

    // Get the current submesh's layout
    pub fn layout(&self) -> VertexLayout {
        self.attributes.layout()
    }
}

impl ToGlName for SubMesh {
    fn name(&self) -> NonZeroU32 {
        self.vao
    }
}

// A mesh is simply a collection of submeshes
pub struct Mesh {
    // Multiple submeshes make up a mesh
    submeshes: Vec<SubMesh>,

    // The full AABB bounds of the mesh
    bounds: AABB,
}

impl Mesh {
    // Create a new empty mesh that can be modified later
    fn new(_ctx: &mut Context) -> Self {
        Self {
            submeshes: Default::default(),
            bounds: todo!(),
        }
    }

    // Create a mesh from multiple submeshes
    fn from_submeshes(_ctx: &mut Context, submeshes: Vec<SubMesh>) -> Self {
        Self { submeshes, bounds: todo!() }
    }

    // Create a mesh that can hold a specific number of submeshes in memory
    fn with_capacity(_ctx: &mut Context, capacity: usize) -> Self {
        Self {
            submeshes: Vec::with_capacity(capacity),
            bounds: todo!(),
        }
    }

    // Insert a submesh into the mesh
    fn insert(&mut self, _ctx: &mut Context, submesh: SubMesh) {
        self.submeshes.push(submesh)
    }
}

impl<'ctx> Asset<'ctx> for Mesh {
    type Args = &'ctx mut Context;

    fn extensions() -> &'static [&'static str] {
        GeometryBuilder::extensions()
    }

    fn deserialize(bytes: assets::loader::CachedSlice, ctx: Self::Args) -> Self {
        let main = GeometryBuilder::deserialize(bytes, ()).build(ctx).unwrap();
        Self::from_submeshes(ctx, vec![main])
    }
}
