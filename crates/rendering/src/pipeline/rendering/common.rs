use std::ptr::null;

use crate::{
    basics::{material::Material, mesh::Mesh, shader::Shader, uniforms::Uniforms},
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
pub(crate) fn render_model(_settings: &RenderingSettings, renderer: &RenderedModel, pipeline: &Pipeline) {
    // Fallback values
    fn fallback_material(pipeline: &Pipeline) -> &Material {
        pipeline.materials.get(&pipeline.defaults().pbr_mat).unwrap()
    }
    fn fallback_shader(pipeline: &Pipeline) -> &Shader {
        pipeline.shaders.get(&pipeline.defaults().shader).unwrap()
    }

    // Render the mesh
    let material = pipeline.materials.get(renderer.material);
    // Load the default material if we don't have a valid one
    let material = material.unwrap_or(fallback_material(pipeline));
    // The shader will always be valid
    let shader = pipeline.shaders.get(&material.shader).unwrap_or_else(|| fallback_shader(pipeline));
    let mesh = pipeline.meshes.get(renderer.mesh).unwrap();

    // Create some uniforms
    let mut uniforms = Uniforms::new(shader.program(), pipeline, true);

    // And set them
    uniforms.set_mat44f32("project_view_matrix", &pipeline.camera().projm_viewm);
    uniforms.set_mat44f32("mesh_matrix", renderer.matrix);
    material.uniforms.execute(&mut uniforms);

    // Finally render the mesh
    unsafe {
        render(mesh);
    }
}

// A normal object that we will render
pub struct RenderedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b vek::Mat4<f32>,

    // Used for rendering
    pub material: &'b Handle<Material>,
}

// A shadowed object that we will render
pub struct ShadowedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b vek::Mat4<f32>,
}
