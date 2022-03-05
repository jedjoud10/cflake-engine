use crate::{
    basics::{
        shader::{load_includes, query_shader_uniforms_definition_map, IncludeExpansionError, ShaderSource, ShaderProgram, ShaderInitSettings, compile_shader},
        uniforms::Uniforms
    },
    pipeline::Pipeline, object::PipelineCollectionElement,
};
use ahash::{AHashSet, AHashMap};
use getset::Getters;
use gl::types::GLuint;
use std::{collections::HashSet, ffi::CString, ptr::null};

use super::ComputeShaderExecutionSettings;

// A compute shader that can run parallel calculations on the GPU
#[derive(Getters)]
pub struct ComputeShader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    program: Option<ShaderProgram>,
    // Init settings
    #[getset(get = "pub")]
    settings: ShaderInitSettings,
}

impl PipelineCollectionElement for ComputeShader {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        // Compiling
        self.program = Some(compile_shader(self.settings.sources()));
    }

    fn disposed(self) {
        todo!()
    }
}

impl ComputeShader {
    // Creates a new compute shader using some shader init settings 
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(settings.sources_mut());
        let (_, source) = sources.iter_mut().nth(0).unwrap();
        let mut included_paths: AHashSet<String> = AHashSet::new();
        // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
        // Include the includables until they cannot be included
        while load_includes(&settings, &mut source.text_mut(), &mut included_paths)? {
            // We are still including paths
        }
        *settings.sources_mut() = sources;
        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: Default::default(),
            settings,
        })
    }
    /*
    // Run a compute shader, and return it's GlTracker
    pub(crate) fn compute_run(&self, pipeline: &Pipeline, settings: ComputeShaderExecutionSettings) -> GlTracker {
        // Create some shader uniforms settings that we can use
        let uniform_settings = ShaderUniformsSettings::new(ShaderIDType::OpenGLID(self.program));
        let uniforms = Uniforms::new(&uniform_settings, pipeline);
        // Dispatch the compute shader for execution
        let axii = settings.axii;

        // Create the GlTracker and send the DispatchCompute command
        GlTracker::new(|| unsafe {
            uniforms.bind_shader();
            // Execute the uniforms
            for x in settings.callbacks {
                x.execute(&uniforms);
            }
            gl::DispatchCompute(axii.x as u32, axii.y as u32, axii.z as u32);
        })
    }
    */
}
