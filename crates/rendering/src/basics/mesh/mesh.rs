use super::Vertices;
use assets::Asset;
use gl::types::GLuint;
use obj::TexturedVertex;
use veclib::{vec2, vec3};

// A simple mesh that holds vertex, normal, and color data
#[derive(Default)]
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
    pub vertices: Vertices,

    // Triangles
    pub indices: Vec<u32>,
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

impl Asset for Mesh {
    // Load a mesh from an asset file
    fn deserialize(mut self, _meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let parsed_obj = obj::load_obj::<TexturedVertex, &[u8], u32>(bytes).unwrap();
        // Generate the tangents
        // Create the actual Mesh now
        for vertex in parsed_obj.vertices {
            self.vertices
                .add()
                .with_position(vec3(vertex.position[0], vertex.position[1], vertex.position[2]))
                .with_normal(vec3((vertex.normal[0] * 127.0) as i8, (vertex.normal[1] * 127.0) as i8, (vertex.normal[2] * 127.0) as i8))
                .with_uv(vec2((vertex.texture[0] * 255.0) as u8, (vertex.texture[1] * 255.0) as u8));
        }
        self.indices = parsed_obj.indices;
        Some(self)
    }
}
