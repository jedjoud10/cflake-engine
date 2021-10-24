use resources::{LoadableResource, ResourceManager};
use ecs::{Component, ComponentID, ComponentInternal};
use crate::{ComplexModel, EntityRenderState, Material, Model, ModelDataGPU, RendererFlags};

// A Multi Material renderer, this renders a Complex Model
pub struct MultiMaterialRenderer {
    pub render_state: EntityRenderState,
    pub complex_model: ComplexModel,
    pub sub_models_gpu_data: Vec<ModelDataGPU>,
    pub materials: Vec<Material>,
    pub flags: RendererFlags,
}

impl Default for MultiMaterialRenderer {
    fn default() -> Self {
        Self {
            render_state: EntityRenderState::Visible,
            complex_model: ComplexModel::default(),
            sub_models_gpu_data: Vec::new(),
            materials: Vec::new(),
            flags: RendererFlags::DEFAULT
        }
    }
}

// Main traits implemented
ecs::impl_component!(MultiMaterialRenderer);

// Everything related to the creation of a Multi Material renderer
impl MultiMaterialRenderer {
    // Create a renderer
    pub fn new() -> Self {
        return Self::default();
    }
    // Load a model into this mm renderer, with a specific material binded to the model
    pub fn load_model(mut self, model_path: &str, material: Material, resource_manager: &mut ResourceManager) -> Self {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::new().from_resource(resource).unwrap();
        self.complex_model.add_model(model);
        self.materials.push(material);
        return self;
    }
    // Enable / disable the wireframe rendering for this entity
    pub fn set_wireframe(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags.insert(RendererFlags::WIREFRAME);
        } else {
            self.flags.remove(RendererFlags::WIREFRAME);
        }
        return self;
    }
}

impl MultiMaterialRenderer {
    // When we update the complex model and want to refresh it's OpenGL data
    pub fn refresh_complex_model(&mut self) {
        // Loop through each sub model and consider it as a unique model
        if self.sub_models_gpu_data.len() != self.complex_model.sub_models.len() {
            panic!();
        }
        self.sub_models_gpu_data = vec![ModelDataGPU::default(); self.complex_model.sub_models.len()];
        for (i, sub_model) in self.complex_model.sub_models.iter().enumerate() {
            let gpu_data = sub_model.refresh_gpu_data();
            self.sub_models_gpu_data[i] = gpu_data;
        }        
    }

    // Dispose of our complex model data
    pub fn dispose_model(&mut self) {
        unsafe {
            for sub_model_gpu_data in self.sub_models_gpu_data.iter_mut() {
                    // Delete the vertex array
                    gl::DeleteBuffers(1, &mut sub_model_gpu_data.vertex_array_object);        
            }
        }
    }
}