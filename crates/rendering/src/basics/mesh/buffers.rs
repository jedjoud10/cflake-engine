use arrayvec::ArrayVec;
use getset::Getters;
use gl::types::GLuint;

use crate::advanced::buffer::Buffer;

// Geometry buffers that contain the underlying OpenGL buffers for our sub-mesh
pub struct MeshBuffers {
    // All the buffers
    inner: ArrayVec<GLuint, 6>,

    // Required
    indices: Buffer<u32>,
    positions: Buffer<vek::Vec3<f32>>,

    // Optional
    normals: Buffer<vek::Vec3<i8>>,
    tangents: Buffer<vek::Vec4<i8>>,
    colors: Buffer<vek::Rgb<u8>>,
    uvs: Buffer<vek::Vec2<u8>>,
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

