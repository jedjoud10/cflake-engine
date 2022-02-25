use super::VertexBuilder;

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
    pub fn reset(&mut self) {
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

    // Create a vertex builder
    pub fn add(&mut self) -> VertexBuilder {
        VertexBuilder { vertices: self }
    }
}
