use super::attributes::{NamedAttribute, AttributeSet};
use crate::{buffer::{Buffer, GPUSendable, RefMapped, MutMapped}, context::Context};
use assets::Asset;
use std::{num::NonZeroU32};

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
    indices: Buffer<u32>,

    // Vertex layout for attributes
    layout: VertexLayout,

    // Can we modify the VAO after we've created it?
    dynamic: bool,
}

impl SubMesh {
    // This creates a new submesh with attribute layout defined by "layout"
    // This will initialize a valid VAO, EBO, and the proper vertex attribute buffers
    pub fn new(ctx: &mut Context, layout: VertexLayout, dynamic: bool) -> Self {
        // Create and bind the VAO, then create a safe VAO wrapper
        let vao = unsafe {
            let mut name = 0;
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            NonZeroU32::new(name).unwrap()
        };

        // Create the sub mesh
        Self {
            vao,
            attributes: AttributeSet::new(vao, ctx, layout, dynamic),
            indices: Buffer::new(ctx, !dynamic),
            vert_count: 0,
            layout,
            dynamic,
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

    // Overwrite the indices internally
    pub fn set_indices(&mut self, ctx: &mut Context, indices: Vec<u32>) {
        self.indices.overwrite(ctx, indices)
    }

    // Insert a vertex set into the sub
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

impl Mesh {
    // Add a submesh into the mesh
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
