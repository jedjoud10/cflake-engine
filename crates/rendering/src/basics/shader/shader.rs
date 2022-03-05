use ahash::{AHashMap, AHashSet};
use gl::types::GLuint;
use crate::basics::shader::{query_shader_uniforms_definition_map, ShaderSourceType, compile_source};
use crate::object::PipelineCollectionElement;
use super::{UniformsDefinitionMap, ShaderSource, ShaderInitSettings, compile_shader};
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::null;

use super::{load_includes, IncludeExpansionError, ShaderProgram};

// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
pub struct Shader {
    // The OpenGL program linked to this shader
    program: ShaderProgram,
    // Init settings
    settings: ShaderInitSettings,
}

// Getters
impl Shader {
    fn program(&self) -> &ShaderProgram { &self.program }
    fn settings(&self) -> &ShaderInitSettings { &self.settings }
}

impl Shader {
    // Creates a new shader using some shader init settings 
    pub fn new(mut settings: ShaderInitSettings) -> Result<Self, IncludeExpansionError> {
        // Loop through the shader sources and edit them
        let mut sources = std::mem::take(settings.sources_mut());
        for (_, source) in sources.iter_mut() {
            let mut included_paths: AHashSet<String> = AHashSet::new();
            // We won't actually generate any subshaders here, so we don't need anything related to the pipeline
            // Include the includables until they cannot be included
            while load_includes(&settings, &mut source.text_mut(), &mut included_paths)? {
                // We are still including paths
            }
        }
        *settings.sources_mut() = sources;

        // Add this shader source to be generated as a subshader
        Ok(Self {
            program: Default::default(),
            settings,
        })
    }
}

impl PipelineCollectionElement for Shader {
    fn added(&mut self, collection: &mut crate::pipeline::PipelineCollection<Self>, handle: crate::pipeline::Handle<Self>) {
        // Compiling
        self.program = compile_shader(self.settings.sources_mut());
    }

    fn disposed(self) {
        todo!()
    }
}