use std::fmt::Debug;
use crate::{
    basics::Buildable,
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
};

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
    pub vertices: Vec<veclib::Vector2<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    pub triangles: Vec<u32>,
}

impl PipelineObject for Model2D {}

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

impl From<math::shapes2d::ShapeType2D> for Model2D {
    fn from(shape: math::shapes2d::ShapeType2D) -> Self {
        // Convert each shape to a model 2D
        match shape {
            math::shapes2d::ShapeType2D::Square(square) => {
                // Create a model2D from a square
                // Create the default vertices first, then we scale them and offset
                let mut vertices = vec![
                    veclib::vec2(-1.0, -1.0),
                    veclib::vec2(-1.0, 1.0),
                    veclib::vec2(1.0, -1.0),
                    veclib::vec2(1.0, 1.0_f32),
                ];
                // Scale then offset
                for vert in vertices.iter_mut() {
                    *vert *= square.size;
                    *vert += square.center;
                }

                Self {
                    vertices,
                    uvs: vec![
                        veclib::vec2(0.0, 0.0),
                        veclib::vec2(0.0, 1.0),
                        veclib::vec2(1.0, 0.0),
                        veclib::vec2(1.0, 1.0),
                    ],
                    colors: Vec::new(),
                    triangles: vec![0, 1, 2, 2, 1, 3],
                }
            },
            math::shapes2d::ShapeType2D::Polygon(polygon) => {
                // Create a model2D from a polygon that contains N vertices
                todo!()
            },
        }
    }
}