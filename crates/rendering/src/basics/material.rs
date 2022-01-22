use crate::basics::*;
use crate::object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask};
use crate::pipeline::*;
use bitflags::bitflags;

use super::shader::Shader;
use super::texture::Texture;
use super::uniforms::ShaderUniformsGroup;

bitflags! {
    pub struct MaterialFlags: u8 {
        const DOUBLE_SIDED = 0b00000001;
    }
}
impl Default for MaterialFlags {
    fn default() -> Self {
        Self::empty()
    }
}

// A material that can have multiple parameters and such
#[derive(Default)]
pub struct Material {
    pub shader: ObjectID<Shader>,
    pub flags: MaterialFlags,
    pub uniforms: ShaderUniformsGroup,
}

impl PipelineObject for Material {}

impl Buildable for Material {
    fn pre_construct(mut self, pipeline: &Pipeline) -> Self {
        let defaults = pipeline.defaults.as_ref().unwrap();
        self.set_pre_construct_settings(defaults.missing_tex, defaults.black, defaults.normals_tex);
        // Set the default rendering shader if no shader was specified
        if !self.shader.valid() {
            self.shader = defaults.shader;
        }
        self
    }

    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.materials.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create the task and send it
        (PipelineTask::CreateMaterial(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

// This should help us create a material
impl Material {
    // Set the main shader
    pub fn set_shader(mut self, shader: ObjectID<Shader>) -> Self {
        self.shader = shader;
        self
    }
    // Add a flag to our flags
    pub fn add_flag(mut self, flag: MaterialFlags) -> Self {
        self.flags.insert(flag);
        self
    }
    // Remove a flag from our flags
    pub fn remove_flag(mut self, flag: MaterialFlags) -> Self {
        self.flags.remove(flag);
        self
    }
    // Set the uniforms
    pub fn set_uniforms(mut self, uniforms: ShaderUniformsGroup) -> Self {
        self.uniforms = uniforms;
        self
    }
    // Set the main diffuse texture
    pub fn set_diffuse_texture(mut self, texture: ObjectID<Texture>) -> Self {
        self.uniforms.set_texture("diffuse_tex", texture, 0);
        self
    }
    // Set the main emissiion texture
    pub fn set_emissive_texture(mut self, texture: ObjectID<Texture>) -> Self {
        self.uniforms.set_texture("emissive_tex", texture, 1);
        self
    }
    // Set the normal map texture
    pub fn set_normals_texture(mut self, texture: ObjectID<Texture>) -> Self {
        self.uniforms.set_texture("normals_tex", texture, 2);
        self
    }

    pub fn set_pre_construct_settings(&mut self, diffuse_tex: ObjectID<Texture>, emissive_tex: ObjectID<Texture>, normals_tex: ObjectID<Texture>) {
        let group = &mut self.uniforms;
        // Create some default uniforms
        group.set_vec2f32("uv_scale", veclib::Vector2::<f32>::ONE);
        group.set_vec3f32("tint", veclib::Vector3::<f32>::ONE);
        group.set_f32("normals_strength", 1.0);
        group.set_f32("emissive_strength", 1.0);
        if !group.contains_uniform("diffuse_tex") {
            group.set_texture("diffuse_tex", diffuse_tex, 0);
        }
        if !group.contains_uniform("emissive_tex") {
            group.set_texture("emissive_tex", emissive_tex, 1);
        }
        if !group.contains_uniform("normals_tex") {
            group.set_texture("normals_tex", normals_tex, 2);
        }
    }
}
