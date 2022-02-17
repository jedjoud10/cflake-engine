// How we store the vertices
#[derive(Default)]
pub struct Vertices {
    // Vertex attribute arrays
    pub positions: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<i8>>,
    pub tangents: Vec<veclib::Vector4<i8>>,
    pub uvs: Vec<veclib::Vector2<u8>>,
    pub colors: Vec<veclib::Vector3<u8>>,
}

impl Vertices {
    pub fn len(&self) -> usize {
        self.positions.len()
    }
}
