use crate::mesh::Mesh;

use super::EnabledAttributes;

// Raw vertex trait that will be implemented for the vertex layout tuples
// This is only used to fetch the buffers, and make sure that there are no attribute collisions
pub trait RawVertexLayout: Sized {
    // The buffers that we will use when pushing/iterating through vertices
    type Buffers;

    // Owned vertex layout
    type Owned;

    // Get the buffers from the mesh
    fn fetch(mesh: &mut Mesh) -> Option<Self::Buffers>;
}

// Main layout trait that allows us to access the mesh vertices
// This is super useful when we try to insert new vertices into the mesh,
// because we can be sure that the vertex attribute buffers all have the same length
pub trait VertexLayout: RawVertexLayout {
    // The vertex layout represented by the enabled mesh buffers
    const ENABLED: EnabledAttributes;

    // Add a single vertex to the buffers
    fn push(buffers: &mut Self::Buffers, vertex: Self);

    // Add a bunch of vertices to the buffers
    fn extend_from_slice(buffers: &mut Self::Buffers, vertices: &[Self]);

    // Remove the last vertex from the buffers
    fn pop(buffers: &mut Self::Buffers) -> Option<Self>;

    // Count the number of vertices that are stored in the buffers
    fn len(buffers: &Self::Buffers) -> usize;
}