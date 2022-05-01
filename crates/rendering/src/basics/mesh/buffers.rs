use arrayvec::ArrayVec;
use getset::Getters;
use gl::types::GLuint;
use crate::advanced::storages::Storage;

// Geometry buffers that contain the underlying OpenGL buffers for our sub-mesh
pub struct MeshBuffers {
    // All the buffers
    inner: ArrayVec<GLuint, 6>,

    // Required
    indices: Storage<u32>,
    positions: Storage<vek::Vec3<f32>>,

    // Optional
    normals: Option<Storage<vek::Vec3<i8>>>,
    tangents: Option<Storage<vek::Vec4<i8>>>,
    colors: Option<Storage<vek::Rgb<u8>>>,
    uvs: Option<Storage<vek::Vec2<u8>>>,
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

