// Multiple vertices and their attributes
#[derive(Default)]
pub struct VertexSet {
    // Positions in 3D
    pub positions: Vec<vek::Vec3<f32>>,
    
    // Normal direction for each vertex
    pub normals: Vec<vek::Vec3<i8>>,

    // Tangents of the normals
    pub tangents: Vec<vek::Vec4<i8>>,
    
    // Texture coordinates for each vertex
    pub uvs: Vec<vek::Vec2<u8>>,

    // Unique vertex color, in case we need it
    pub colors: Vec<vek::Rgb<u8>>,
}

impl VertexSet {
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}
