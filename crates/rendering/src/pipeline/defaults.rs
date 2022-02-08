use crate::{
    basics::{lights::LightSource, material::Material, model::Model, shader::Shader, texture::Texture},
    object::ObjectID,
};

// Some default values like the default material or even the default shader
pub struct DefaultPipelineObjects {
    pub missing_tex: ObjectID<Texture>,
    pub black: ObjectID<Texture>,
    pub white: ObjectID<Texture>,
    pub normals_tex: ObjectID<Texture>,
    pub shader: ObjectID<Shader>,
    pub material: ObjectID<Material>,
    pub model: ObjectID<Model>,
    // This value might change, since the user might remove the directional light
    pub sun: ObjectID<LightSource>,
}
