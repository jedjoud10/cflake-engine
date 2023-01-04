use std::mem::MaybeUninit;

use graphics::{VertexBuffer, Vertex};

#[cfg(not(feature = "two-dim"))]
bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledAttributes: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORDS = 1 << 4;
    }
}

#[cfg(feature = "two-dim")]
bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledAttributes: u8 {
        const POSITIONS = 1;
        const COLORS = 1 << 3;
    }
}

// This is the maximum number of active attributes that we can have inside a mesh
pub const MAX_MESH_VERTEX_ATTRIBUTES: usize =
    EnabledAttributes::all().bits.trailing_ones() as usize;

// Contains the underlying array buffer for a specific attribute
pub type AttributeBuffer<A> = MaybeUninit<VertexBuffer<<<A as VertexAttribute>::V as Vertex>::Storage>>;


// A named attribute that has a specific name, like "Position", or "Normal"
pub trait VertexAttribute {
    type V: Vertex;

    /*
    // Get the proper reference from the wrapper vertex types
    fn from_vertices_ref_as_ref<'a>(vertices: &'a VerticesRef) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_ref<'a>(vertices: &'a VerticesMut) -> &'a AttributeBuffer<Self>;
    fn from_vertices_mut_as_mut<'a>(vertices: &'a mut VerticesMut) -> &'a mut AttributeBuffer<Self>;

    // Insert an attribute buffer into the vertices
    fn insert(vertices: &mut VerticesMut, buffer: ArrayBuffer<Self::Out>);

    // Remove an attribute from the vertices
    fn remove(vertices: &mut VerticesMut);
    */
}