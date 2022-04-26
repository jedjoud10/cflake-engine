use crate::{
    basics::{
        shader::{compile_shader, load_includes, IncludeExpansionError, ShaderInitSettings, Program, PreCompilationData, SharedExpansionData},
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
    program: Option<Program>,

    // Pre-compilation data
    pre: Option<PreCompilationData>,
}

impl ObjectSealed for ComputeShader {
    fn init(&mut self, _pipeline: &mut Pipeline) {
        self.program = Some(compile_shader(self.pre.take().unwrap()));
    }

    fn disposed(self) {
        unsafe {
            gl::DeleteProgram(self.program.as_ref().unwrap().name());
        }
    }
}

impl ComputeShader {
    // Creates a new compute shader using some shader init settings
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Get the first source available, since compute shaders only have one shader source
        let mut sources = std::mem::take(settings.sources_mut());
        let (_, source) = sources.iter_mut().next().unwrap();

        // Data that keeps track of was was expanded and what wasn't
        let mut shared = SharedExpansionData::default();

        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        while load_includes(&settings, source.text_mut(), &mut shared)? {
            // We are still including paths
        }
        
        // Create the pre-compilation data 
        let pre = PreCompilationData {
            sources,
            shared,
        };

        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: None,
            pre: Some(pre),
        })
    }
    // Execute a compute shader
    // This makes 100% sure that we are already bound to the compute shader, since we require the uniforms
    pub fn run(&self, _pipeline: &Pipeline, settings: ComputeShaderExecutionSettings, _uniforms: Uniforms, force: bool) -> Result<(), OpenGLObjectNotInitialized> {
        // Bruh bruh bruh
        if self.program().is_none() {
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
