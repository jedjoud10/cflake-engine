// Vertex attribute layout that we can use to read/modify vertices of a mesh
pub trait VertLayout {
    // Tuple contains safe mutable references to the underlying vertex data
    type Tuple;

    // Make sure the vertex layout is safe
    fn verify() -> bool;

    // Try to get the tuple at a specific index
    // PS: This assumes that the tuple is already valid for this layout
    fn get(index: usize) -> Self::Tuple;
}




// Multiple vertices and their attributes
#[derive(Default)]
pub struct VertexSet {
    // Positions in 3D
    positions: Vec<vek::Vec3<f32>>,
    
    // Normal direction for each vertex
    normals: Vec<vek::Vec3<i8>>,

    // Tangents of the normals
    tangents: Vec<vek::Vec4<i8>>,
    
    // Texture coordinates for each vertex
    uvs: Vec<vek::Vec2<u8>>,

    // Unique vertex color, in case we need it
    colors: Vec<vek::Rgb<u8>>,

    // Number of vertices we have in total
    len: usize,
}

impl VertexSet {
}