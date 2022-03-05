#[cfg(test)]
pub mod tests {
    use crate::basics::mesh::Mesh;

    // Test mesh generation
    #[test]
    fn mesh() {
        // Geometry Builder
        let mut mesh = Mesh::default();
        let builder = mesh.builder();
        let mut vbuilder = builder.vertex_builder;
        // A single vertex lol
        vbuilder
            .position(veclib::Vector3::ONE)
            .color(veclib::Vector3::ZERO);
    }
}