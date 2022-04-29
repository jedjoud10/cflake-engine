use super::{IndexBuilder, Mesh, VertexBuilder, MeshBuffers};

// Some arbitrary shape in 3D
// This geometry must ALWAYS be valid
pub struct Geometry<Attributes: VertAttrib> {
    // Underlying buffers
    buffers: MeshBuffers,

    // The vertices and their attributes that make up the geometry
    vertices: VertexSet,

    // How we connect the vertices to each other (triangles)
    indices: IndexSet,

    // Geometry flags 
}