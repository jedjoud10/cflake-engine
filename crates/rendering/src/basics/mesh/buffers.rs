use arrayvec::ArrayVec;
use getset::Getters;
use gl::types::GLuint;

use crate::advanced::storages::TypedStorage;

// Mesh buffers
#[derive(Getters)]
#[getset(get = "pub")]
pub struct MeshBuffers {
    // All the buffers
    pub(crate) inner: ArrayVec<GLuint, 6>,

    // Required
    pub(crate) indices: TypedStorage<u32>,
    pub(crate) positions: TypedStorage<veclib::Vector3<f32>>,

    // Optional
    pub(crate) normals: Option<TypedStorage<veclib::Vector3<i8>>>,
    pub(crate) tangents: Option<TypedStorage<veclib::Vector4<i8>>>,
    pub(crate) colors: Option<TypedStorage<veclib::Vector3<u8>>>,
    pub(crate) uvs: Option<TypedStorage<veclib::Vector2<u8>>>,
    /*
    pub element_buffer_object: u32,

    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,

    pub color_buf: u32,
    pub uv_buf: u32,
    */
}

impl Drop for MeshBuffers {
    // Dipose of the mesh buffers
    fn drop(&mut self) {
        // Delete the VBOs
        unsafe {
            gl::DeleteBuffers(self.inner.len() as i32, self.inner.as_ptr());
        }
    }
}
