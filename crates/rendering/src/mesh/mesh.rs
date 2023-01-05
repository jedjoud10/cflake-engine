use graphics::IndexBuffer;

use crate::{AttributeBuffer, EnabledMeshAttributes};
use crate::mesh::attributes::{TexCoord, Color, Tangent, Normal, Position};

// A mesh is a collection of 3D vertices connected by triangles
#[cfg(not(feature = "two-dim"))]
pub struct Mesh {
    // Enabled mesh attributes
    enabled: EnabledMeshAttributes,

    // Vertex attribute buffers
    positions: AttributeBuffer<Position>,
    normals: AttributeBuffer<Normal>,
    tangents: AttributeBuffer<Tangent>,
    colors: AttributeBuffer<Color>,
    uvs: AttributeBuffer<TexCoord>,

    // The number of vertices stored in this mesh
    len: usize,

    // The triangle buffer
    triangles: IndexBuffer<u32>,
}

// A mesh is a collection of 2D vertices connected by triangles
#[cfg(feature = "two-dim")]
pub struct Mesh {
    // Enabled mesh attributes
    enabled: EnabledMeshAttributes,

    // Vertex attribute buffers
    positions: AttributeBuffer<Position>,
    colors: AttributeBuffer<Color>,

    // The number of vertices stored in this mesh
    len: usize,

    // The triangle buffer
    triangles: IndexBuffer<u32>,
}