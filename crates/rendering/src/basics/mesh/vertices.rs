// How we store the vertices
#[derive(Default)]
pub struct Vertices {
    // Vertex attribute arrays
    pub positions: Vec<vek::Vec3<f32>>,
    pub normals: Vec<vek::Vec3<i8>>,
    pub tangents: Vec<vek::Vec4<i8>>,
    pub uvs: Vec<vek::Vec2<u8>>,
    pub colors: Vec<vek::Rgb<u8>>,
}

impl Vertices {
    // Length and is_empty
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

#[derive(Default)]
// A vertex builder that helps us create multiple vertices and add them to the mesh
pub struct VertexBuilder {
    pub vertices: Vertices,
}

impl VertexBuilder {
    // Le builder pattern
    pub fn position(&mut self, position: vek::Vec3<f32>) {
        self.vertices.positions.push(position);
    }
    pub fn normal(&mut self, normal: vek::Vec3<i8>) {
        self.vertices.normals.push(normal);
    }
    pub fn tangent(&mut self, tangent: vek::Vec4<i8>) {
        self.vertices.tangents.push(tangent);
    }
    pub fn uv(&mut self, uv: vek::Vec2<u8>) {
        self.vertices.uvs.push(uv);
    }
    pub fn color(&mut self, color: vek::Rgb<u8>) {
        self.vertices.colors.push(color);
    }
}
