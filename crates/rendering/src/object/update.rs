use super::{ObjectID, PipelineObject};
use crate::{
    advanced::{atomic::AtomicGroup, compute::ComputeShader, shader_storage::ShaderStorage},
    basics::{material::Material, model::Model, renderer::Renderer, shader::Shader, texture::Texture},
    pipeline::{camera::Camera, Pipeline, PipelineRenderer},
};

// Task that we will send to the pipeline whenever we want to update a specific pipeline object
pub enum UpdateTask {
    UpdateRendererMatrix(ObjectID<Renderer>, veclib::Matrix4x4<f32>),
    UpdateCamera(Camera),
    UpdateWindowDimensions(veclib::Vector2<u16>),
    UpdateWindowFocus(bool),
}

impl UpdateTask {
    // Execute the update task
    pub(crate) fn execute(self, pipeline: &mut Pipeline, renderer: &mut PipelineRenderer) {
        match self {
            UpdateTask::UpdateRendererMatrix(id, matrix) => {
                let renderer = pipeline.get_renderer_mut(id).unwrap();
                renderer.update_matrix(matrix);
            }
            UpdateTask::UpdateCamera(x) => pipeline.set_internal_camera(x),
            UpdateTask::UpdateWindowDimensions(x) => pipeline.update_window_dimensions(renderer, x),
            UpdateTask::UpdateWindowFocus(x) => pipeline.update_window_focus_state(x),
        }
    }
}
