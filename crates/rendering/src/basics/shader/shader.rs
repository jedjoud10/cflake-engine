use ahash::AHashMap;
use getset::Getters;
use gl::types::GLuint;
use crate::basics::shader::query_shader_uniforms_definition_map;
use crate::basics::uniforms::UniformsDefinitionMap;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::null;

use super::{load_includes, IncludeExpansionError, ShaderProgram};

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
#[derive(Getters)]
pub struct Shader {
    // The OpenGL program linked to this shader
    #[getset(get = "pub")]
    pub(crate) program: ShaderProgram,
    // The updated and modified shader sources
    #[getset(get = "pub")]
    pub(crate) sources: AHashMap<String, ShaderSource>,
    // Uniforms definition map
    #[getset(get = "pub")]
    pub(crate) uniforms: UniformsDefinitionMap,
}