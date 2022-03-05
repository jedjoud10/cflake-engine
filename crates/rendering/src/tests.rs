#[cfg(test)]
pub mod tests {
    use crate::basics::{mesh::Mesh, shader::{ShaderInitSettingsBuilder, ShaderSource}};

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

    // Builder test
    #[test]
    fn builder() {
        // Shader builder load thingy
        let settings = ShaderInitSettingsBuilder::default().directive_const("lol", "bozo").build();
    }
}