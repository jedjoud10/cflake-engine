use std::{ptr::null, cell::Ref};

use crate::{basics::{mesh::Mesh, material::Material, shader::Shader, uniforms::{Uniforms, StoredUniforms}}, pipeline::{Handle, Pipeline}};

use super::RenderingSettings;

// Render a simple mesh
// This assumes that the proper matrices, uniforms, and textures have been set before this call
pub(crate) unsafe fn render(mesh: &Mesh) {
    // Don't render if the mesh is invalid
    if mesh.vao() != 0 {
        // Actually draw the mesh
        gl::BindVertexArray(mesh.vao());
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.buffers()[0]);
        gl::DrawElements(gl::TRIANGLES, mesh.indices().len() as i32, gl::UNSIGNED_INT, null());    
    }
}

// Render a model
pub(crate) fn render_model(settings: RenderingSettings, renderer: RenderedModel, pipeline: &Pipeline) {
    // Fallback values
    fn fallback_material(pipeline: &Pipeline) -> Ref<Material> {
        pipeline.materials.get(&pipeline.defaults().material).unwrap()
    }
    fn fallback_shader(pipeline: &Pipeline) -> Ref<Shader> {
        pipeline.shaders.get(&pipeline.defaults().shader).unwrap()
    }
    fn fallback_mesh(pipeline: &Pipeline) -> Ref<Mesh> {
        pipeline.meshes.get(&pipeline.defaults().mesh).unwrap()
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
    uniforms.set_mat44f32("project_view_matrix", settings.camera.projm_viewm);
    uniforms.set_mat44f32("mesh_matrix", renderer.matrix);
    // Optional
    material.uniforms.execute(&mut uniforms);
    renderer.uniforms.execute(&mut uniforms);
    // Textures might be not valid, so we fallback to the default ones just in case
    uniforms.set_texture("diffuse_tex", &material.textures.diffuse_map, 0);
    uniforms.set_texture("normals_tex", &material.textures.normal_map, 1);
    uniforms.set_texture("emissive_tex", &material.textures.emissive_map, 2);
    uniforms.set_vec3f32("tint", material.tint);
    uniforms.set_f32("normals_strength", material.normal_map_strength);
    uniforms.set_f32("emissive_strength", material.emissive_map_strength);
    uniforms.set_vec2f32("uv_scale", material.uv_scale);

    // Finally render the mesh
    unsafe {
        render(&mesh);
    }
}

// A normal object that we will render
pub struct RenderedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b veclib::Matrix4x4<f32>,

    // Used for rendering
    pub material: &'b Handle<Material>,

    // Extra uniforms if we want
    pub uniforms: &'b StoredUniforms,
}

// A shadowed object that we will render
pub struct ShadowedModel<'b> {
    // Required
    pub mesh: &'b Handle<Mesh>,
    pub matrix: &'b veclib::Matrix4x4<f32>,
}