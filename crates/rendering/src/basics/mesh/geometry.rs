use opengl::types::GLuint;
use crate::advanced::buffer::Buffer;

// Some arbitrary shape in 3D
// This geometry must ALWAYS be valid
pub struct Geometry {
    // Main VAO that contains all of our buffers
    vao: GLuint,

    // Main vertex attributes
    positions: Buffer<u32>,
    normals: Buffer<vek::Vec3<i8>>,
    tangents: Buffer<vek::Vec4<i8>>,
    colors: Buffer<vek::Rgb<u8>>,
    texcoords: Buffer<vek::Vec2<u8>>,    

    // How we connect the vertices to each other (triangles)
    indices: Buffer<u32>,
}