use crate::basics::*;
use crate::object::{PipelineObject, PipelineTask, ObjectID, ObjectBuildingTask};
use crate::pipeline::*;
use bitflags::bitflags;

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
    fn pre_construct(self, pipeline: &Pipeline) -> Self {
        // Create some default uniforms
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2f32("uv_scale", veclib::Vector2::<f32>::ONE);
        group.set_vec3f32("tint", veclib::Vector3::<f32>::ONE);
        group.set_f32("normals_strength", 1.0);
        if !group.contains_uniform("diffuse_tex") { group.set_texture("diffuse_tex", pipeline.defaults.unwrap().diffuse_tex, 0); }
        if !group.contains_uniform("normals_tex") { group.set_texture("normals_tex", pipeline.defaults.unwrap().normals_tex, 1); }
        self.uniforms = group;
        // Set the default rendering shader if no shader was specified
        if !self.shader.valid() { self.shader = pipeline.defaults.unwrap().shader }
        self
    }

    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.materials.get_next_idx_increment();
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
}