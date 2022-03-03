use ahash::AHashMap;
use gl::types::GLuint;
use crate::basics::shader::query_shader_uniforms_definition_map;
use crate::basics::uniforms::UniformsDefinitionMap;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::null;

use super::{load_includes, IncludeExpansionError};

// Shader source type
pub(crate) enum ShaderSourceType {
    Vertex,
    Fragment,
    Compute,
}
// And a shader source
pub(crate) struct ShaderSource {
    // Corresponding path for this shader source, since we store them in different files
    pub path: String,
    // The actual source code text
    pub text: String,
    // And a specific type just to help use
    pub _type: ShaderSourceType,
}
// A shader that contains just some text sources that it loaded from the corresponding files, and it will send them to the Render Thread so it can actually generate the shader using those sources
pub struct Shader {
    // The OpenGL program linked to this shader
    pub(crate) program: GLuint,
    // The updated and modified shader sources
    pub(crate) sources: AHashMap<String, ShaderSource>,
    // Uniforms definition m,ap
    pub(crate) uniforms: UniformsDefinitionMap,
}

impl Shader {
    // Creates a shader from it's corresponding shader settings
    pub fn new(mut settings: ShaderSettings) -> Result<Self, IncludeExpansionError> {
        // Create "self"
        let mut shader = Self {
            program: 0,
            sources: HashMap::default(),
        };
        
        Ok(shader)
    }
}
