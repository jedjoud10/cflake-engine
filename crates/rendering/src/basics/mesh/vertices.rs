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
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    // Reset all the buffers
    pub(crate) fn reset(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.tangents.clear();
        self.uvs.clear();
        self.colors.clear();
        self.positions.shrink_to_fit();
        self.normals.shrink_to_fit();
        self.tangents.shrink_to_fit();
        self.uvs.shrink_to_fit();
        self.colors.shrink_to_fit();
    }
}

#[derive(Default)]
// A vertex builder that helps us create multiple vertices and add them to the mesh
pub struct VertexBuilder {
    pub vertices: Vertices,
}

impl VertexBuilder {
    // Le builder pattern
    pub fn position(&mut self, position: veclib::Vector3<f32>) {
        self.vertices.positions.push(position);
    }
    pub fn normal(&mut self, normal: veclib::Vector3<i8>) {
        self.vertices.normals.push(normal);
    }
    pub fn tangent(&mut self, tangent: veclib::Vector4<i8>) {
        self.vertices.tangents.push(tangent);
    }
    pub fn uv(&mut self, uv: veclib::Vector2<u8>) {
        self.vertices.uvs.push(uv);
    }
    pub fn color(&mut self, color: veclib::Vector3<u8>) {
        self.vertices.colors.push(color);
    }
}
