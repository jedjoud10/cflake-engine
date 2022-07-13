use std::mem::MaybeUninit;

use crate::buffer::ArrayBuffer;

use super::{Attribute, Mesh};

// Contains the underlying array buffer for a specific attribute
type AttribBuffer<A> = MaybeUninit<ArrayBuffer<<A as Attribute>::Out>>;

bitflags::bitflags! {
    // This specifies the buffers that the mesh uses internally
    pub struct EnabledVertexBuffers: u8 {
        const POSITIONS = 1;
        const NORMALS = 1 << 1;
        const TANGENTS = 1 << 2;
        const COLORS = 1 << 3;
        const TEX_COORD = 1 << 4;
        const INDICES = 1 << 5;
    }
}


// Main layout trait that allows us to access the mesh vertices
// This is super useful when we try to insert new vertices into the mesh,
// because we can be sure that the vertex attribute buffers all have the same length
pub trait VertexLayout {
    // The vertex layout represented by the enabled mesh buffers
    const ENABLED: EnabledVertexBuffers;

    // The buffers that we will use when pushing/iterating through vertices
    type Buffers;

    // The owned version of this vertex layout that we will push
    type OwnedIn;

    // Get the buffers from the mesh
    fn fetch(mesh: &mut Mesh) -> Self::Buffers;

    // Push a new vertex into the fetched buffers
    fn push(buffers: &mut Self::Buffers, vertex: Self::OwnedIn);
}