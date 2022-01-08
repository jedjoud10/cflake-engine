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
pub struct Material {
    shader: Option<ObjectID<Shader>>, // The shader that we will use to render this material
    flags: MaterialFlags, // The special flags that this material has that changes how it is rendered
    uniforms: ShaderUniformsGroup, // A uniform group specific for this material
}

impl PipelineObject for Material {}

impl Buildable for Material {
    fn pre_construct(self, pipeline: &Pipeline) -> Self {
        // Create some default uniforms
        let mut group = ShaderUniformsGroup::new();
        group.set_vec2f32("uv_scale", veclib::Vector2::<f32>::ONE);
        group.set_vec3f32("tint", veclib::Vector3::<f32>::ONE);
        group.set_f32("normals_strength", 1.0);
        group.set_texture("diffuse_tex", pipeline.default_diffuse_tex, 0);
        group.set_texture("normals_tex", pipeline.default_normals_tex, 1);
        self.uniforms = group;
        // Set the default rendering shader if no shader was specified
        self.shader.get_or_insert(pipeline.default_shader);
        self
    }

    fn construct(self, pipeline: &Pipeline) -> ObjectID<Self> {
        // Create the ID
        let id = pipeline.materials.get_next_idx_increment();
        let id = ObjectID::new(id);
        // Create the task and send it
        crate::pipec::task(PipelineTask::CreateMaterial(ObjectBuildingTask::<Self>(self, id)), pipeline);
        id
    }
}

// This should help us create a material
impl Material {
    // Set the main shader
    pub fn set_shader(mut self, shader: ObjectID<Shader>) -> Self {
        self.shader = Some(shader);
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
}