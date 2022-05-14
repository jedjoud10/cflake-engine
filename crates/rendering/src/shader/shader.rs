use crate::context::Cached;

use super::{FragmentStage, VertexStage};

// A shader that will render our objects onto the screen
// This will make use of two shader programs, the vertex programs, and fragment program
pub struct Shader {
    // Shader source that is ran for every vertex
    vertex: VertexStage,

    // Shader source that is ran for every visible fragment in the viewport1
    fragment: FragmentStage,
}

impl Cached for Shader {}
