use std::ptr::null;

use math::bounds::aabb::AABB;

use crate::{
    basics::{material::Material, mesh::Mesh, uniforms::Uniforms},
    pipeline::{Handle, Pipeline},
};

// Render a simple mesh
// This assumes that the proper matrices, uniforms, and textures have been set before this call
pub(crate) unsafe fn render(mesh: &Mesh) {
    // Don't render if the mesh is invalid
    if mesh.vao() != 0 {
        // Actually draw the mesh
        gl::BindVertexArray(mesh.vao());
        let indices = mesh.buffers().as_ref().unwrap().indices().buffer();
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);
        gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, null());
    }
}

// A normal object that we will render
pub struct RenderedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b vek::Mat4<f32>,

    // Certified frustum culling moment
    pub aabb: &'b AABB,

    // Used for rendering
    pub material: &'b Handle<Material>,
}

// The last model that we have drawn
pub struct LastRenderedModelInfo<'b> {
    pub material: &'b Handle<Material>,
}
// A shadowed object that we will render
pub struct ShadowedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b vek::Mat4<f32>,
}
