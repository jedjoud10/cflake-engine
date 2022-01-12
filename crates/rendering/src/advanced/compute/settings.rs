use crate::{object::ObjectID, ShaderUniformsGroup};

use super::{ComputeShader, ComputeShaderTask};

// Some compute shader settings that we can use whenever we want to execute a compute shader
pub struct ComputeShaderExecutionSettings {
    // We must have the ID of the compute shader
    pub(crate) id: ObjectID<ComputeShader>,
    // We must know the axii groups
    pub(crate) axii: (u16, u16, u16),
    // Some tasks that we will execute after executing the compute shader
    pub(crate) tasks: Vec<ComputeShaderTask>,
    // Store some shader uniforms, if we want to
    pub(crate) uniforms: Option<ShaderUniformsGroup>,
}

impl ComputeShaderExecutionSettings {
    // Create some new compute shader execution settings
    pub fn new(id: ObjectID<ComputeShader>, axii: (u16, u16, u16)) -> Self {
        Self {
            id,
            axii,
            tasks: Vec::new(),
            uniforms: None,
        }
    }
    // We can also specify some tasks to execute, but this is optional
    pub fn task(mut self, task: ComputeShaderTask) -> Self {
        self.tasks.push(task);
        self
    }
    // Set the uniforms
    pub fn set_uniforms(mut self, uniforms: ShaderUniformsGroup) -> Self {
        self.uniforms = Some(uniforms);
        self
    }
}
