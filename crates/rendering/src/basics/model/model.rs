use crate::{object::{PipelineObject, ObjectID, ConstructionTask, Construct}, utils::DataType, pipeline::Pipeline};
use std::fmt::Debug;
use super::{CustomVertexDataBuffer, StoredCustomVertexDataBuffer};

// Some OpenGL data for a model
#[derive(Default, Debug)]
pub struct ModelBuffers {
    // The OpenGL data
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub tangent_buf: u32,
    pub uv_buf: u32,
    pub color_buf: u32,
    // Some custom vertex data if we want
    pub custom_vertex_data: u32,

    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub triangle_count: usize,
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
    pub(crate) custom: Option<StoredCustomVertexDataBuffer>,

    // Triangles
    pub triangles: Vec<u32>,
}

impl PipelineObject for Model {
    // Reserve an ID for this model
    fn reserve(self, pipeline: &Pipeline) -> Option<(Self, ObjectID<Self>)> where Self: Sized {
        Some((self, ObjectID::new(pipeline.models.get_next_id_increment())))
    }
    // Send this model to the pipeline for construction
    fn send(self, pipeline: &Pipeline, id: ObjectID<Self>) -> ConstructionTask {
        ConstructionTask::Material(Construct::<Self>(self, id))
    }
    // Add the model to our ordered vec
    fn add(mut self, pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<()> where Self: Sized {
        // Add the model
        pipeline.models.insert(id.get()?, self);
        Some(())
    }
    // Remove the model from the pipeline
    fn delete(pipeline: &mut Pipeline, id: ObjectID<Self>) -> Option<Self> where Self: Sized {
        pipeline.models.remove(id)
    }
}


impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("vertices", &self.vertices.len())
            .field("normals", &self.normals.len())
            .field("tangents", &self.tangents.len())
            .field("uvs", &self.uvs.len())
            .field("colors", &self.colors.len())
            .field("triangles", &self.triangles.len())
            .finish()
    }
}

impl Model {
    // Create a new model with some certain capacity to hold a number of vertices
    pub fn with_capacity(vertices_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices_count),
            normals: Vec::with_capacity(vertices_count),
            tangents: Vec::with_capacity(vertices_count),
            uvs: Vec::with_capacity(vertices_count),
            colors: Vec::with_capacity(vertices_count),
            custom: None,
            triangles: Vec::with_capacity(vertices_count * 3),
        }
    }
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    pub fn combine(mut self, other: Self) -> Self {
        // We must have matching custom vertex buffers
        if self.custom.is_some() != other.custom.is_some() {
            panic!()
        };

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
        if let Some(custom) = other.custom {
            self.custom.as_mut().unwrap().inner.extend(custom.inner.into_iter());
        }
        self
    }
    // Combine a model with this one
    // NOTE: This assumes that the second model uses vertices from the first model
    pub fn combine_smart(mut self, other: Self) -> Self {
        // We must have matching custom vertex buffers
        if self.custom.is_some() != other.custom.is_some() {
            panic!()
        };

        self.triangles.extend(other.triangles.into_iter());
        self.vertices.extend(other.vertices.into_iter());
        self.normals.extend(other.normals.into_iter());
        self.uvs.extend(other.uvs.into_iter());
        self.colors.extend(other.colors.into_iter());
        self.tangents.extend(other.tangents.into_iter());
        self
    }
    // Procedurally generate the normals for this model
    pub fn generate_normals(mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles

        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; self.vertices.len()];
        for i in 0..(self.triangles.len() / 3) {
            // Calculate the face normal
            let (i1, i2, i3) = (self.triangles[i * 3], self.triangles[i * 3 + 1], self.triangles[i * 3 + 2]);
            // Get the actual vertices
            let a = self.vertices.get(i1 as usize).unwrap();
            let b = self.vertices.get(i2 as usize).unwrap();
            let c = self.vertices.get(i3 as usize).unwrap();

            // Calculate
            let d1 = b - a;
            let d2 = c - a;
            let cross = veclib::Vector3::<f32>::cross(d1, d2).normalized();

            // Add the face normal to our local vertices
            vertex_normals[i1 as usize] += cross;
            vertex_normals[i2 as usize] += cross;
            vertex_normals[i3 as usize] += cross;
        }

        // Now we must normalize
        for vertex_normal in vertex_normals.iter_mut() {
            vertex_normal.normalize();
        }

        // Update our normals
        self.normals = vertex_normals;
    }
    // Add some custom vertex data
    pub fn with_custom<T>(mut self, custom: CustomVertexDataBuffer<T>, _type: DataType) -> Self {
        self.custom = Some(StoredCustomVertexDataBuffer::new(custom, _type));
        self
    }
}
