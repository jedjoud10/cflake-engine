use std::fmt::Debug;

use veclib::{Vector, VectorElemCount};

use crate::{
    basics::Buildable,
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
};

use super::{CustomVertexDataBuffer, StoredCustomVertexDataBuffer};

// Some OpenGL data for a 2D model
#[derive(Default, Debug)]
pub struct Model2DBuffers {
    // The OpenGL data
    pub vertex_buf: u32,
    pub uv_buf: u32,
    pub color_buf: u32,
    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub triangle_count: usize,
}

impl PipelineObject for Model2DBuffers {}

// Basically a 2D model that will be rendered to the screen using some 2D shaders 
pub struct Model2D {
    // Per vertex data
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub triangles: Vec<u32>,
}

impl PipelineObject for Model2D {}

impl Buildable for Model2D {
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.materials.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create the task and send it
        (PipelineTask::CreateModel2D(ObjectBuildingTask::<Self>(self, id)), id)
    }
}

impl Debug for Model2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("vertices", &self.vertices.len())
            .field("uvs", &self.uvs.len())
            .field("colors", &self.colors.len())
            .field("triangles", &self.triangles.len())
            .finish()
    }
}

impl From<math::shapes::>
