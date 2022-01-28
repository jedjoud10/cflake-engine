use std::{fmt::Debug, any::TypeId};

use veclib::{VectorElemCount, Vector};

use crate::{
    object::{ObjectBuildingTask, ObjectID, PipelineObject, PipelineTask},
    pipeline::Pipeline,
    basics::Buildable,
};

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

impl PipelineObject for ModelBuffers {}

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

impl PipelineObject for Model {}

impl Buildable for Model {
    fn construct_task(self, pipeline: &Pipeline) -> (PipelineTask, ObjectID<Self>) {
        // Create the ID
        let id = pipeline.materials.get_next_id_increment();
        let id = ObjectID::new(id);
        // Create the task and send it
        (PipelineTask::CreateModel(ObjectBuildingTask::<Self>(self, id)), id)
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
    // Combine a model with this one
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
    // Procedurally generate the normals for this model
    pub fn generate_normals(&mut self) {
        // First, loop through every triangle and calculate it's face normal
        // Then loop through every vertex and average out the face normals of the adjacent triangles  

        let mut vertex_normals: Vec<veclib::Vector3<f32>> = vec![veclib::Vector3::ZERO; self.vertices.len()];
        for i in 0..(self.triangles.len()/3) {
            // Calculate the face normal
            let (i1, i2, i3) = (self.triangles[i*3], self.triangles[i*3+1], self.triangles[i*3+2]);
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
    pub fn set_custom<T, U: Vector<T> + VectorElemCount>(&mut self, custom_vertex_buffer: CustomVertexDataBuffer<T, U>) {
        // We gotta serialize the data now, in native endian
        let ptr = custom_vertex_buffer.inner.as_ptr();
        let byte_size = std::mem::size_of::<U>();
        let total_len = byte_size * custom_vertex_buffer.inner.len();
        let slice: &[u8] = unsafe { std::slice::from_raw_parts(ptr as *const u8, total_len) };
        let vec = slice.to_vec();
        self.custom = Some(
            StoredCustomVertexDataBuffer {
                inner: vec,
                size_per_component: U::ELEM_COUNT,
                _type: custom_vertex_buffer._type,
            }
        )
    }
}
