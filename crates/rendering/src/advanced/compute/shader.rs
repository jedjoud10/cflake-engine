use crate::{
    basics::{
        shader::{compile_shader, load_includes, IncludeExpansionError, ShaderInitSettings, ShaderProgram},
        uniforms::Uniforms,
    },
    object::{ObjectSealed, OpenGLObjectNotInitialized},
    pipeline::Pipeline,
};
use ahash::AHashSet;
use getset::Getters;

use super::ComputeShaderExecutionSettings;

// A compute shader that can run parallel calculations on the GPU
#[derive(Getters)]
pub struct ComputeShader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    program: ShaderProgram,
    // Init settings
    #[getset(get = "pub")]
    settings: ShaderInitSettings,
}

impl ObjectSealed for ComputeShader {
    fn init(&mut self, _pipeline: &mut Pipeline) {
        self.program = compile_shader(self.settings.sources());
    }

    fn disposed(self) {
        unsafe {
            gl::DeleteProgram(self.program.program());
        }
    }
}

impl ComputeShader {
    // Creates a new compute shader using some shader init settings
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(settings.sources_mut());
        let (_, source) = sources.iter_mut().next().unwrap();
        let mut included_paths: AHashSet<String> = AHashSet::new();
        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        while load_includes(&settings, source.text_mut(), &mut included_paths)? {
            // We are still including paths
        }
        *settings.sources_mut() = sources;
        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: Default::default(),
            settings,
        })
    }
    // Execute a compute shader
    pub fn run(&self, _pipeline: &Pipeline, settings: ComputeShaderExecutionSettings, _uniforms: Uniforms, force: bool) -> Result<(), OpenGLObjectNotInitialized> {
        // Check validity
        if self.program().program() == 0 {
            return Err(OpenGLObjectNotInitialized);
        }

        // Run
        unsafe {
            let axii = settings.axii;
            // TODO: fix this
            if force {
                gl::MemoryBarrier(gl::BUFFER_UPDATE_BARRIER_BIT | gl::ATOMIC_COUNTER_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT);
            }
            gl::DispatchCompute(axii.x as u32, axii.y as u32, axii.z as u32);
            if force {
                gl::Finish();
                gl::MemoryBarrier(gl::BUFFER_UPDATE_BARRIER_BIT | gl::ATOMIC_COUNTER_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT);
            }
        }

        Ok(())
    }
}
