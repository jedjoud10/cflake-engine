use std::ptr::null;

use crate::{
    basics::{material::Material, mesh::Mesh, uniforms::Uniforms},
    pipeline::{Handle, Pipeline},
};

use super::RenderingSettings;

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

// Render a model
pub(crate) fn render_model<'a>(_settings: &RenderingSettings, renderer: &RenderedModel<'a>, last_mat_handle: &mut Handle<Material>, pipeline: &Pipeline) {
    // Load the default material if we don't have a valid one
    let mat_handle = renderer.material.fallback_to(&pipeline.defaults().pbr_mat);
    let mat = pipeline.materials.get(mat_handle).unwrap();
    // However, if we have an invalid shader, we must panic
    let shader = pipeline.shaders.get(&mat.shader).unwrap();
    let mesh = pipeline.meshes.get(renderer.mesh).unwrap();

    // Create some uniforms
    let mut uniforms = Uniforms::new(shader.program(), pipeline);

    // Set the uniforms
    uniforms.set_mat44f32("mesh_matrix", renderer.matrix);

    // Check if we really need to set the material uniforms
    if *last_mat_handle != mat_handle.clone() {
        uniforms.set_mat44f32("project_view_matrix", &pipeline.camera().projm_viewm);
        mat.uniforms.execute(&mut uniforms);
        *last_mat_handle = renderer.material.clone();
    }

    // Finally render the mesh
    unsafe {
        render(mesh);
    }

    // Set last material used
    *last_mat_handle = renderer.material.clone();
}

// A normal object that we will render
pub struct RenderedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b vek::Mat4<f32>,

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
