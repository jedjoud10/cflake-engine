#[cfg(test)]
pub mod tests {
    use crate::{
        advanced::atomic::AtomicGroup,
        basics::{
            mapper::MappableGLBuffer,
            material::{Material, MaterialTextures},
            mesh::Mesh,
            shader::{Directive, ShaderInitSettings},
        },
        pipeline::Handle,
        utils::{AccessType, UpdateFrequency, UsageType},
    };

    // Test mesh generation
    #[test]
    fn mesh() {
        // Geometry Builder
        let mut mesh = Mesh::default();
        let builder = mesh.modifier();
        let mut vbuilder = builder.vertex_builder;
        // A single vertex lol
        vbuilder.position(veclib::Vector3::ONE).color(veclib::Vector3::ZERO);
    }

    // Builder test
    #[test]
    fn builder() {
        // Shader builder load thingy
        let settings = ShaderInitSettings::default().directive("lol", Directive::Const("cock".to_string()));
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
