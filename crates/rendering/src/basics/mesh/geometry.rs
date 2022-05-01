use super::{MeshBuffers, TriangleSet, VertexSet};

// Some arbitrary shape in 3D
// This geometry must ALWAYS be valid
#[derive(Default)]
pub struct Geometry {
    // Underlying buffers
    buffers: MeshBuffers,

    // The vertices and their attributes that make up the geometry
    vertices: VertexSet,

    // How we connect the vertices to each other (triangles)
    triangles: TriangleSet,
}