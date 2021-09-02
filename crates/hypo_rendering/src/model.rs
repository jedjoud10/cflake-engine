use hypo_resources::Resource;
use hypo_resources::ResourceManager;

// A simple model that holds vertex, normal, and color data
#[derive(Default, Debug, Clone)]
pub struct Model {
    pub vertices: Vec<veclib::Vector3<f32>>,
    pub normals: Vec<veclib::Vector3<f32>>,
    pub tangents: Vec<veclib::Vector4<f32>>,
    pub uvs: Vec<veclib::Vector2<f32>>,
    pub triangles: Vec<u32>,
}

impl Model {
    // Turns a loaded resource model into an actual model
    pub fn from_resource(resource: &Resource) -> Option<Self> {
        match resource {
            Resource::Model(model) => {
                // Turn the loaded model into a normal model
                let new_model = Self {
                    vertices: model.vertices.clone(),
                    normals: model.normals.clone(),
                    tangents: model.tangents.clone(),
                    uvs: model.uvs.clone(),
                    triangles: model.indices.clone(),
                };
                Some(new_model)
            }
            _ => None,
        }
    }
    // Loads a model directly from a path
    pub fn load_model(path: &str, resource_manager: &mut ResourceManager) -> Option<Self> {
        let resource = resource_manager.load_packed_resource(path)?;
        return Self::from_resource(resource);
    }
    // Flip all the triangles in the mesh, basically making it look inside out. This also flips the normals
    pub fn flip_triangles(&mut self) {
        for i in (0..self.triangles.len()).step_by(3) {
            // Swap the first and last index of each triangle
            self.triangles.swap(i, i + 2);
        }
    }
    // Combine a model with this one, and return the new model
    // TODO: Implement vertex merging while combining models
    pub fn combine(&self, other: &Self) -> Self {
        let mut output_model = self.clone();
        let max_triangle_index: u32 = self.vertices.len() as u32;
        // Get the max triangle inde
        let mut final_tris = other.triangles.clone();
        for x in final_tris.iter_mut() {
            *x += max_triangle_index;
        }
        output_model.triangles.extend(final_tris);
        output_model.vertices.extend(other.vertices.clone());
        output_model.normals.extend(other.normals.clone());
        output_model.uvs.extend(other.uvs.clone());
        output_model.tangents.extend(other.tangents.clone());
        return output_model;
    }
}

// Struct that hold the model's information from OpenGL
#[derive(Default, Debug)]
pub struct ModelDataGPU {
    pub vertex_buf: u32,
    pub normal_buf: u32,
    pub uv_buf: u32,
    pub tangent_buf: u32,
    pub vertex_array_object: u32,
    pub element_buffer_object: u32,
    pub initialized: bool,
}

// Trait that allows you to make procedural models
pub trait ProceduralModelGenerator {
    // Generate the model
    fn generate_model(&self) -> Model;
}
