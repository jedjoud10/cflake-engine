use crate::object::{Construct, ConstructionTask, Deconstruct, DeconstructionTask, ObjectID, PipelineObject};
use crate::pipeline::*;

use super::shader::Shader;
use super::texture::Texture;
use super::uniforms::{SetUniformsCallback, Uniforms};
// A material that can have multiple parameters and such
pub struct Material {
    // Main settings
    pub shader: ObjectID<Shader>,
    pub(crate) uniforms: SetUniformsCallback,

    // Actual parameters used for rendering
    pub diffuse_map: ObjectID<Texture>,
    pub normal_map: ObjectID<Texture>,
    pub emissive_map: ObjectID<Texture>,
    pub tint: veclib::Vector3<f32>,
    pub normal_map_strength: f32,
    pub emissive_map_strength: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            shader: Default::default(),
            uniforms: Default::default(),
            diffuse_map: Default::default(),
            normal_map: Default::default(),
            emissive_map: Default::default(),
            tint: veclib::Vector3::ONE,
            normal_map_strength: 1.0,
            emissive_map_strength: 1.0,
        }
    }
}

impl PipelineObject for Material {
    // Reserve an ID for this material
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> {
        Some((self, pipeline.materials.gen_id()))
    }
    // Send this material to the pipeline for construction
    fn send(self, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Material(Construct::<Self>(self, id))
    }
    // Create a deconstruction task
    fn pull(id: ObjectID<Self>) -> DeconstructionTask {
        DeconstructionTask::Material(Deconstruct::<Self>(id))
    }
    // Add the material to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> {
        // Make sure we have a valid shader
        if !self.shader.is_some() {
            self.shader = pipeline.defaults.as_ref()?.shader;
        }

        // Add the material
        pipeline.materials.insert(id, self)?;
        Some(())
    }
    // Remove the material from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> {
        pipeline.materials.remove(id)
    }
}

// This should help us create a material
impl Material {
    // Set the main shader
    pub fn with_shader(mut self, shader: ObjectID<Shader>) -> Self {
        self.shader = shader;
        self
    }
    // Set the uniforms callback
    pub fn with_uniforms(mut self, callback: SetUniformsCallback) -> Self {
        self.uniforms = callback;
        self
    }
    // With some parameters
    // Maps
    pub fn with_diffuse(mut self, diffuse_map: ObjectID<Texture>) -> Self {
        self.diffuse_map = diffuse_map;
        self
    }
    pub fn with_normal(mut self, normal_map: ObjectID<Texture>) -> Self {
        self.normal_map = normal_map;
        self
    }
    pub fn with_emissive(mut self, emissive_map: ObjectID<Texture>) -> Self {
        self.emissive_map = emissive_map;
        self
    }
    // Values
    pub fn with_normal_strength(mut self, strength: f32) -> Self {
        self.normal_map_strength = strength;
        self
    }
    pub fn with_emissive_strenhgth(mut self, strength: f32) -> Self {
        self.emissive_map_strength = strength;
        self
    }
    pub fn with_tint(mut self, tint: veclib::Vector3<f32>) -> Self {
        self.tint = tint;
        self
    }
}
