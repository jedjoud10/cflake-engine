use crate::{object::{PipelineObject, ObjectID, PipelineTask, ObjectBuildingTask}, Buildable};

// Some OpenGL data for a model
#[derive(Default)]
pub(crate) struct ModelBuffers {
    // The OpenGL data
    pub(crate) vertex_buf: u32,
    pub(crate) normal_buf: u32,
    pub(crate) tangent_buf: u32,
    pub(crate) uv_buf: u32,
    pub(crate) color_buf: u32,

    pub(crate) vertex_array_object: u32,
    pub(crate) element_buffer_object: u32,
}

// A simple model that holds vertex, normal, and color data
#[derive(Default)]
pub struct Model {
    // Per vertex data
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub colors: Vec<veclib::Vector3<f32>>,
    // Triangles
    pub triangles: Vec<u32>,
    // Some model buffers    
    pub(crate) buffers: Option<ModelBuffers>,
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Self { 
            vertices: self.vertices.clone(),
            normals: self.normals.clone(),
            tangents: self.tangents.clone(),
            uvs: self.uvs.clone(),
            colors: self.colors.clone(),
            triangles: self.triangles.clone(),
            buffers: None
        }
    }
}

impl PipelineObject for Model {}

impl Buildable for Model {
    fn construct(self, pipeline: &crate::Pipeline) -> crate::object::ObjectID<Self> {
        // Create the ID
        let id = pipeline.materials.get_next_idx_increment();
        let id = ObjectID::new(id);
        // Create the task and send it
        crate::pipec::task(PipelineTask::CreateModel(ObjectBuildingTask::<Self>(self, id)), pipeline);
        id
    }
}

impl Model {
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    pub fn combine(mut self, other: Self) -> Self {
        let max_triangle_index: u32 = self.vertices.len() as u32;
        // Get the max triangle inde
        let mut final_tris = other.triangles.clone();
        for x in final_tris.iter_mut() {
            *x += max_triangle_index;
        }
        self.triangles.extend(final_tris);
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        self
    }
    // Comebine a model with this one
    // NOTE: This assumes that the second model uses vertices from the first model
    pub fn combine_smart(mut self, other: Self) -> Self {
        self.triangles.extend(other.triangles.into_iter());
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        self
    }
}
