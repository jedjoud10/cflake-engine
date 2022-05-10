use super::{attributes::{AttributeSet, NamedAttribute, vertex::*}, GeometryBuilder};
use crate::{
    buffer::{Buffer, MutMapped, RefMapped, ElementBuffer, BufferAccess},
    context::Context,
};
use assets::Asset;
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
    // The VAO that wraps everything up (OpenGL side)
    vao: NonZeroU32,

    // Vertex attributes and the vertex count
    attributes: AttributeSet,
    vert_count: usize,

    // We must always have a valid EBO
    indices: ElementBuffer,

    // Vertex layout for attributes
    layout: VertexLayout,
}

impl SubMesh {

    // Construct a submesh using a geometry builder
    // This creates a new submesh with attribute layout defined by the builder itself
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    pub fn new(ctx: &mut Context, builder: GeometryBuilder) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            NonZeroU32::new(name).unwrap()
        };

        // How we shall access the vertex attribute and index buffers
        let access = BufferAccess::WRITE | BufferAccess::WRITE;

        // Create the sub mesh using the builder's layout and attributes
        let layout = builder.layout();
        Self {
            vao,
            attributes: AttributeSet::new(vao, ctx, layout, access),
            indices: Buffer::from_vec(ctx, access, builder.indices),
            vert_count: 0,
            layout,
        }
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
}


impl Asset for Mesh {
    type OptArgs = ();

    fn is_valid(meta: assets::metadata::AssetMetadata) -> bool {
        meta.extension() == "obj"
    }

    unsafe fn deserialize(_bytes: &[u8], _args: &Self::OptArgs) -> Option<Self> {
        todo!()
    }
}
