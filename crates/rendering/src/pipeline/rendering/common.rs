use std::ptr::null;

use crate::basics::mesh::Mesh;

// Render a single mesh
// This assumes that the proper matrices, uniforms, and textures have been set before this call
pub(crate) fn render(mesh: &Mesh) {
    unsafe {
        // Actually draw the mesh
        gl::BindVertexArray(mesh.vao());
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.buffers()[0]);
        gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, null());
    }
}
