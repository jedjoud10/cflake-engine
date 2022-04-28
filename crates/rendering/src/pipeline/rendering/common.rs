use std::ptr::null;

use math::bounds::aabb::AABB;

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

// Le oui oui rendering?
pub(crate) fn render_model<'a>(renderer: RenderedModel<'a>, pipeline: &Pipeline) {
    // Load the default missing material if we don't have a valid one
    let handle = renderer.material.fallback_to(&pipeline.defaults().missing_pbr_mat);
    let material = pipeline.get(handle).unwrap();

    // However, if we have an invalid shader, we must panic
    let shader = pipeline.get(material.shader().as_ref().unwrap()).unwrap();
    let mesh = pipeline.get(renderer.mesh).unwrap();
    
    // Create some uniforms
    Uniforms::new(shader.program(), pipeline, |mut uniforms| {
        // Set the camera uniforms
        let camera = pipeline.camera();
        uniforms.set_mat44f32("_pv_matrix", &camera.proj_view);
        uniforms.set_vec2f32("_nf_planes", camera.clips);
        uniforms.set_vec3f32("_cam_pos", camera.position);
        uniforms.set_vec3f32("_cam_dir", camera.forward);        

        // Set the model uniforms
        uniforms.set_mat44f32("_model_matrix", renderer.matrix);

        // Execute the material 
        material.execute(pipeline, uniforms);
    });

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
