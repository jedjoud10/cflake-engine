use crate::basics::{Material, Model, ModelDataGPU};
use assets::{Asset, AssetManager};

// A Multi Material renderer, this renders a Complex Model
pub struct MultiMaterialRenderer {
    pub sub_models: Vec<(Model, usize)>,
    pub sub_models_gpu_data: Vec<ModelDataGPU>,
    pub materials: Vec<Material>,
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
        Self::default()
    }
    // Load a model into this mm renderer, with a specific material binded to the model
    pub fn load_model(mut self, model_path: &str, material: Material, asset_manager: &AssetManager) -> Option<Self> {
        let model = Model::default().load_asset(model_path, &asset_manager.asset_cacher)?;
        self.sub_models.push((model, self.materials.len()));
        self.materials.push(material);
        Some(self)
    }
    // Add a specific model to the complex model as a submodel
    pub fn add_submodel(mut self, model: Model, material: Material) -> Self {
        self.sub_models.push((model, self.materials.len()));
        self.materials.push(material);
        self
    }
    // Set the materials
    pub fn set_materials(mut self, materials: Vec<Material>) -> Self {
        self.materials = materials;
        self
    }
    // Add a specific mode to the complex model as a submodel, but without it's material
    pub fn add_submodel_m(mut self, model: Model, material_id: usize) -> Self {
        self.sub_models.push((model, material_id));
        self
    }
}

impl MultiMaterialRenderer {
    // When we update the complex model and want to refresh it's OpenGL data
    pub fn refresh_sub_models(&mut self) {
        // Loop through each sub model and consider it as a unique model
        self.sub_models_gpu_data = vec![ModelDataGPU::default(); self.sub_models.len()];
        for (i, (sub_model, _material_id)) in self.sub_models.iter().enumerate() {
            let gpu_data = sub_model.refresh_gpu_data();
            self.sub_models_gpu_data[i] = gpu_data;
        }
    }

    // Dispose of our complex model data
    pub fn dispose_models(&mut self) {
        for sub_model_gpu_data in self.sub_models_gpu_data.iter_mut() {
            sub_model_gpu_data.dispose();
        }
    }
}
