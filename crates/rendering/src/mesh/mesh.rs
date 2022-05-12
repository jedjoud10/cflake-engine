use super::{
    attributes::{AttributeSet, NamedAttribute},
    GeometryBuilder,
};
use crate::{
    buffer::{Buffer, BufferAccess, ElementBuffer, MutMapped, RefMapped},
    context::{Context, Cached},
};
use assets::{Asset, loader::AssetBytes};
use std::num::NonZeroU32;

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

// A submesh is a collection of 3D vertices connected by triangles
// Each sub-mesh is associated with a single material
pub struct SubMesh {
    vao: NonZeroU32,
    attributes: AttributeSet,
    vert_count: usize,
    indices: ElementBuffer,
    layout: VertexLayout,
}

impl SubMesh {
    // Construct a submesh using a geometry builder
    // This creates a new submesh with attribute layout defined by the builder itself
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    // PS: This doesn't check if the builder contains different length-vectors
    pub(super) unsafe fn new_unchecked(ctx: &mut Context, mut builder: GeometryBuilder) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            NonZeroU32::new(name).unwrap()
        };

        // How we shall access the vertex attribute and index buffers
        let access = BufferAccess::WRITE | BufferAccess::WRITE;

        // Only take the indices from the builder, cause we store them in a different place than the vertex attributes
        let indices = std::mem::take(&mut builder.indices);
        let layout = builder.layout();

        Self {
            vao,
            attributes: AttributeSet::new(vao, ctx, access, builder),
            indices: Buffer::from_vec(ctx, access, indices),
            vert_count: 0,
            layout,
        }
    }

    // Get the current submesh's layout
    pub fn layout(&self) -> VertexLayout {
        self.layout
    }

    // Get a mapped buffer for a specific vertex attribute, if possible
    pub fn get<U: NamedAttribute>(&self, ctx: &Context) -> Option<RefMapped<U::Out>> {
        U::get(&self.attributes).map(|buffer| buffer.try_map_range(ctx, 0..self.vert_count).unwrap())
    }

    // Get a mutable mapped buffer for a specifc vertex attribute, if possible
    pub fn get_mut<U: NamedAttribute>(&mut self, ctx: &mut Context) -> Option<MutMapped<U::Out>> {
        let count = self.vert_count;
        U::get_mut(&mut self.attributes).map(|buffer| buffer.try_map_range_mut(ctx, 0..count).unwrap())
    }
}

// A mesh is simply a collection of submeshes
pub struct Mesh {
    submeshes: Vec<SubMesh>,
}

impl Mesh {
    // Create a new empty mesh that can be modified later
    fn new(_ctx: &mut Context) -> Self {
        Self { submeshes: Default::default() }
    }

    // Create a mesh from multiple submeshes
    fn from_submeshes(_ctx: &mut Context, submeshes: Vec<SubMesh>) -> Self {
        Self { submeshes }
    }

    // Insert a submesh into the mesh
    fn insert(&mut self, _ctx: &mut Context, submesh: SubMesh)  {

    }
}

impl<'ctx> Asset<'ctx> for Mesh {
    type Args = &'ctx mut Context;

    fn is_extension_valid(extension: &str) -> bool {
        extension == "obj"
    }

    fn deserialize(bytes: AssetBytes, args: Self::Args) -> Self {
        todo!()
    }
}