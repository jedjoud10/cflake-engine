use crate::{
    AlbedoMap, CameraUniform, DynamicPipeline, Material, MaterialId,
    NormalMap, Pipeline, SceneUniform, TimingUniform,
};
use ahash::AHashMap;
use assets::Assets;
use bytemuck::Zeroable;
use graphics::{
    BufferMode, BufferUsage, GpuPod, Graphics, LoadOp, Normalized,
    Operation, PipelineInitializationError, RenderPass, StoreOp,
    SwapchainFormat, Texture2D, UniformBuffer, BGRA, RGBA,
};
use std::{
    any::TypeId, cell::RefCell, marker::PhantomData,
    mem::ManuallyDrop, rc::Rc,
};

// A pipeline manager will store and manager multiple material pipelines and their IDs
pub struct Pipelines {
    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynamicPipeline>>,
}

impl Pipelines {
    // Create a new pipeline manager with no stored pipelines
    pub fn new() -> Self {
        Self {
            pipelines: Default::default(),
        }
    }

    // Register a new material pipeline within the renderer
    pub fn register<M: Material>(
        &mut self,
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Result<MaterialId<M>, PipelineInitializationError> {
        // Initialize the pipeline and register it if needed
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            log::debug!(
                "Creating pipeline for material {}...",
                utils::pretty_type_name::<M>()
            );
            let pipeline = Pipeline::<M>::new(graphics, assets)?;
            self.pipelines.insert(key, Rc::new(pipeline));
            log::debug!(
                "Registered pipeline for material {}",
                utils::pretty_type_name::<M>()
            );
        }

        // Material ID is just a marker type for safety
        Ok(MaterialId(PhantomData))
    }

    // Get a MaterialID from a pre-initialized pipeline
    pub fn get<M: Material>(&self) -> Option<MaterialId<M>> {
        let key = TypeId::of::<M>();
        self.pipelines.get(&key).map(|_| MaterialId(PhantomData))
    }

    // Extract the internally stored material pipelines
    pub(crate) fn extract_pipelines(
        &self,
    ) -> Vec<Rc<dyn DynamicPipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }
}
