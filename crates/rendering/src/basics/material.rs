use crate::object::{ObjectID, PipelineObject, ConstructionTask, Construct};
use crate::pipeline::*;

use super::shader::Shader;
use super::texture::Texture;
use super::uniforms::ShaderUniformsGroup;
// A material that can have multiple parameters and such
#[derive(Default)]
pub struct Material {
    pub shader: ObjectID<Shader>,
    pub double_sided: bool,
    pub uniforms: ShaderUniformsGroup,
}

impl PipelineObject for Material {
    // Reserve an ID for this material
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, ObjectID::new(pipeline.materials.get_next_id_increment())))
    }
    // Send this material to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Material(Construct::<Self>(self, id))
    }
    // Add the material to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Some default uniforms
        let mut group = ShaderUniformsGroup::default();
        group.set_vec2f32("uv_scale", veclib::Vector2::<f32>::ONE);
        group.set_vec3f32("tint", veclib::Vector3::<f32>::ONE);
        group.set_f32("normals_strength", 1.0);
        group.set_f32("emissive_strength", 1.0);
        // Make sure we have valid textures in case we don't
        if !self.uniforms.contains_uniform("diffuse_tex") {
            group.set_texture("diffuse_tex", pipeline.defaults?.missing_tex, 0);
        }
        if !self.uniforms.contains_uniform("emissive_tex") {
            group.set_texture("emissive_tex", pipeline.defaults?.black, 1);
        }
        if !self.uniforms.contains_uniform("normals_tex") {
            group.set_texture("normals_tex", pipeline.defaults?.normals_tex, 2);
        }
        // Combine the default uniforms and the new uniforms that we just made
        self.uniforms = ShaderUniformsGroup::combine(self.uniforms, group);

        // Make sure we have a valid shader
        if !self.shader.is_some() { self.shader = pipeline.defaults?.shader; }

        // Add the material
        pipeline.materials.insert(id.get()?, self);
        Some(())
    }
    // Remove the material from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.materials.remove(id.get()?)
    }
}

// This should help us create a material
impl Material {
    // Set the main shader
    pub fn set_shader(mut self, shader: ObjectID<Shader>) -> Self {
        self.shader = shader;
        self
    }
    // Set the uniforms group
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
    // Set the UV scale
    pub fn set_uv_scale(mut self, uv_scale: veclib::Vector2<f32>) -> Self {
        self.uniforms.set_vec2f32("uv_scale", uv_scale);
        self
    }
    // Set the tint (Color)
    pub fn set_tint(mut self, tint: veclib::Vector3<f32>) -> Self {
        self.uniforms.set_vec3f32("tint", tint);
        self
    }
    // Set the normal map's strength
    pub fn set_normals_strength(mut self, strength: f32) -> Self {
        self.uniforms.set_f32("normals_strength", strength);
        self
    }
    // Set the emissive map's strength
    pub fn set_emissive_strength(mut self, strength: f32) -> Self {
        self.uniforms.set_f32("emissive_strength", strength);
        self
    }
}
