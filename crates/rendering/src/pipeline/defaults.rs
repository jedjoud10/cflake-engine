use crate::{
    basics::{
        lights::LightSource, material::Material, mesh::Mesh, shader::Shader, texture::Texture,
    },
    object::ObjectID,
};

// Some default values like the default material or even the default shader
pub struct DefaultPipelineObjects {
    // Textures
    pub missing_tex: ObjectID<Texture>,
    pub black: ObjectID<Texture>,
    pub white: ObjectID<Texture>,
    pub normals_tex: ObjectID<Texture>,

    // Shader
    pub shader: ObjectID<Shader>,

    // Materials
    pub material: ObjectID<Material>,

    // Meshes
    pub mesh: ObjectID<Mesh>,
    pub plane: ObjectID<Mesh>,
    pub cube: ObjectID<Mesh>,
    pub sphere: ObjectID<Mesh>,

    // Other
    // This value might change, since the user might remove the directional light
    pub sun: ObjectID<LightSource>,
}
