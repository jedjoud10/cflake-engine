use std::{ffi::CString, ptr::null};

use crate::GPUObject;

// Sub shader type
#[derive(Debug, Copy, Clone)]
pub enum SubShaderType {
    Vertex,
    Fragment,
    Compute,
}

impl Default for SubShaderType {
    fn default() -> Self {
        Self::Vertex
    }
}

// A sub shader, could be a geometry, vertex, or fragment shader
#[derive(Clone, Default)]
pub struct SubShader {
    pub name: String,
    pub source: String,
    pub subshader_type: SubShaderType,
}

impl SubShader {
    // Compile the current subshader's source code
    pub fn compile_subshader(&mut self) {
        
    }
}
