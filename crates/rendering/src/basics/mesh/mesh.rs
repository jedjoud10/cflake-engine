use super::Vertices;
use gl::types::GLuint;
use obj::TexturedVertex;
use veclib::{vec2, vec3};
use getset::Getters;

// A simple mesh that holds vertex, normal, and color data
#[derive(Getters)]
pub struct Mesh {
    // Main IDs
    pub(crate) vertex_array_object: GLuint,

    // Vertex attributes IDs
    pub(crate) buffers: [GLuint; 6],
    /*
    pub element_buffer_object: u32,

    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,

    pub color_buf: u32,
    pub uv_buf: u32,
    */
    // Store the vertices (in multiple bufer or in a single big buffer)
    #[getset(get = "pub")]
    vertices: Vertices,

    // Triangles
    #[getset(get = "pub")]
    indices: Vec<u32>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        // Dispose of the OpenGL buffers
        unsafe {
            // Delete the VBOs
            gl::DeleteBuffers(self.buffers.len() as i32, self.buffers.as_ptr());

            // Delete the vertex array
            if gl::IsVertexArray(self.vertex_array_object) {
                gl::DeleteVertexArrays(1, &self.vertex_array_object);
            }
        }
    }
}