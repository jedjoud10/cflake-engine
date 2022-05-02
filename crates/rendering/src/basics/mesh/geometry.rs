use opengl::types::GLuint;
use crate::{advanced::buffer::{Buffer}, utils::{UpdateFrequency, AccessType, BufferHints}};


// Some arbitrary shape in 3D
// This geometry must ALWAYS be valid
pub struct Geometry {
    // Main VAO that contains all of our buffers
    vao: GLuint,

    // 3D positions that we will use for rendering
    positions: Buffer<u32>,

    // How we connect the vertices to each other (triangles)
    indices: Buffer<u32>,
}

impl Geometry {
}