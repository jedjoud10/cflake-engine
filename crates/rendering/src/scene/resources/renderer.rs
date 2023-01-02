use std::{any::TypeId, rc::Rc};
use ahash::AHashMap;
use assets::Assets;
use crate::{DynamicPipeline, Material, MaterialId, Pipeline};

// Main resource that will contain data to render objects on the screen
// This will contain the current swapchain texture that we must render to
#[derive(Default)]
pub struct ForwardRenderer {    
    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynamicPipeline>>
}

impl ForwardRenderer {
    // Register a new material pipeline within the renderer
    pub fn register<M: Material>(
        &mut self,
        assets: &Assets,
    ) -> MaterialId<M> {
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            // Load the material shader
            let shader = todo!();

            // Initialize the pipeline and register it
            let pipeline = Pipeline::<M>::new();
            self.pipelines.insert(key, Rc::new(pipeline));
        }

        // Material ID is just a marker type for safety
        MaterialId(Default::default())
    }

    // Get a MaterialID from a pre-initialized pipeline
    pub fn get<M: Material>(&self) -> Option<MaterialId<M>> {
        let key = TypeId::of::<M>();
        self.pipelines
            .get(&key)
            .map(|_| MaterialId(Default::default()))
    }

    // Extract the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn DynamicPipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }
}