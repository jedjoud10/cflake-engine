use resources::{LoadableResource, ResourceManager};
use crate::{Material, Model, ModelDataGPU};

// A Multi Material renderer, this renders a Complex Model
pub struct MultiMaterialRenderer {
    pub sub_models: Vec<Model>,
    pub sub_models_gpu_data: Vec<ModelDataGPU>,
    pub materials: Vec<Option<Material>>,
}

impl Default for MultiMaterialRenderer {
    fn default() -> Self {
        Self {
            sub_models: Vec::new(),
            sub_models_gpu_data: Vec::new(),
            materials: Vec::new(),
        }
    }
}

// Everything related to the creation of a Multi Material renderer
impl MultiMaterialRenderer {
    // Create a renderer
    pub fn new() -> Self {
        return Self::default();
    }
    // Load a model into this mm renderer, with a specific material binded to the model
    pub fn load_model(mut self, model_path: &str, material: Option<Material>, resource_manager: &mut ResourceManager) -> Self {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::new().from_resource(resource).unwrap();
        self.sub_models.push(model);
        self.materials.push(material);
        return self;
    }
    // Add a specific model to the complex model as a submodel
    pub fn add_submodel(mut self, model: Model, material: Option<Material>) -> Self {
        self.sub_models.push(model);
        self.materials.push(material);
        return self;
    }
}

impl MultiMaterialRenderer {
    // When we update the complex model and want to refresh it's OpenGL data
    pub fn refresh_sub_models(&mut self) {
        // Loop through each sub model and consider it as a unique model
        if self.materials.len() != self.sub_models.len() {
            panic!();
        }
        self.sub_models_gpu_data = vec![ModelDataGPU::default(); self.sub_models.len()];
        for (i, sub_model) in self.sub_models.iter().enumerate() {
            let gpu_data = sub_model.refresh_gpu_data();
            self.sub_models_gpu_data[i] = gpu_data;
        }        
    }

    // Dispose of our complex model data
    pub fn dispose_models(&mut self) {
        unsafe {
            for sub_model_gpu_data in self.sub_models_gpu_data.iter_mut() {
                    // Delete the vertex array
                    gl::DeleteBuffers(1, &mut sub_model_gpu_data.vertex_array_object);        
            }
        }
    }
}