use super::{model::Model, model::ModelDataGPU, Material, RendererFlags};
use ecs::{Component, ComponentID, ComponentInternal};
use resources::{LoadableResource, ResourceManager};
// A component that will be linked to entities that are renderable
pub struct Renderer {
    pub render_state: EntityRenderState,
    pub gpu_data: ModelDataGPU,
    pub model: Model,
    // This renderer can only have one material for now (TODO: Make a multi material system)
    pub material: Option<Material>,
    // Flags
    pub flags: RendererFlags,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            render_state: EntityRenderState::Visible,
            gpu_data: ModelDataGPU::default(),
            model: Model::default(),
            material: None,
            flags: RendererFlags::DEFAULT,
        }
    }
}

// Main traits implemented
ecs::impl_component!(Renderer);

// Everything related to the creation of a renderer
impl Renderer {
    // Create a renderer
    pub fn new() -> Self {
        return Self::default().set_material(Material::default());
    }
    // Load a model
    pub fn load_model(mut self, model_path: &str, resource_manager: &mut ResourceManager) -> Self {
        let resource = resource_manager.load_packed_resource(model_path).unwrap();
        let model = Model::new().from_resource(resource).unwrap();
        self.model = model;
        return self;
    }
    // Set a model
    pub fn set_model(mut self, model: Model) -> Self {
        self.model = model;
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
    // With a specific material
    pub fn set_material(mut self, material: Material) -> Self {
        self.material = Some(material);
        return self;
    }
}

impl Renderer {    
    // When we update the model and want to refresh it's OpenGL data
    pub fn refresh_model(&mut self) {        
    }
    // Dispose of our model
    pub fn dispose_model(&mut self) {
        unsafe {
            // Delete the vertex array
            gl::DeleteBuffers(1, &mut self.gpu_data.vertex_buf);
        }
    }
}

// The current render state of the entity
#[derive(Debug)]
pub enum EntityRenderState {
    Visible,
    Invisible,
}

// If the entity is culled or not
#[derive(Debug)]
pub enum EntityCullingState {
    Culled,
    Unculled,
}

impl Default for EntityRenderState {
    fn default() -> Self {
        Self::Visible
    }
}
