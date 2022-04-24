// Settings that control the functionality when we create a new mesh out of vertices and indices
pub struct MeshSettings {
    pub generate_aabb: bool,
}


impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            generate_aabb: false,
        }
    }
}