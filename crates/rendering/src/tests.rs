#[cfg(test)]
pub mod tests {
    use crate::{basics::{mesh::Mesh, shader::{ShaderInitSettingsBuilder, ShaderSource, Directive}, material::{Material, MaterialTextures}}, pipeline::Handle};

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
        let settings = ShaderInitSettingsBuilder::default().directive("lol", Directive::Const("cock".to_string())).build();
    }

    // Material test
    #[test]
    fn material() {
        let mat = Material {
            textures: MaterialTextures {
                diffuse_map: Handle::default(),
                ..Default::default()
            },
            ..Default::default()
        };
    }
}