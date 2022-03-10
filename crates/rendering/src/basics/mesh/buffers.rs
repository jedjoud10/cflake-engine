use arrayvec::ArrayVec;
use getset::Getters;
use gl::types::GLuint;
use smallvec::SmallVec;

use crate::advanced::raw::storage::Storage;

// Mesh buffers
#[derive(Getters)]
#[getset(get = "pub")]
pub struct MeshBuffers {
    // All the buffers
    pub(crate) inner: ArrayVec<GLuint, 6>,

    // Required
    pub(crate) indices: Storage<u32>,
    pub(crate) positions: Storage<veclib::Vector3<f32>>,

    // Optional
    pub(crate) normals: Option<Storage<veclib::Vector3<i8>>>,
    pub(crate) tangents: Option<Storage<veclib::Vector4<i8>>>,
    pub(crate) colors: Option<Storage<veclib::Vector3<u8>>>,
    pub(crate) uvs: Option<Storage<veclib::Vector2<u8>>>,
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
