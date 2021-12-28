// A simple model that holds vertex, normal, and color data
#[derive(Clone)]
pub struct Model {
    pub name: String,
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub triangles: Vec<u32>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            name: crate::pipeline::rname("model"),
            vertices: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

impl Model {
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    pub fn combine(mut self, other: Self) -> Self {
        let max_triangle_index: u32 = self.vertices.len() as u32;
        // Get the max triangle inde
        let mut final_tris = other.triangles.clone();
        for x in final_tris.iter_mut() {
            *x += max_triangle_index;
        }
        self.triangles.extend(final_tris);
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        // Update the name as well
        self.name = format!("{}_{}", self.name, other.name);
        self
    }
    // Comebine a model with this one
    // NOTE: This assumes that the second model uses vertices from the first model
    pub fn combine_smart(mut self, other: Self) -> Self {
        self.triangles.extend(other.triangles.into_iter());
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        // Update the name as well
        self.name = format!("{}_{}", self.name, other.name);
        self
    }
}
