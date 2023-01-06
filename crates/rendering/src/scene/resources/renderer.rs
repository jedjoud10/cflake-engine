use crate::{DynamicPipeline, Material, MaterialId, Pipeline};
use ahash::AHashMap;
use assets::Assets;
use graphics::{Graphics, Normalized, RenderPass, Texture2D, BGRA};
use std::{
    any::TypeId, marker::PhantomData, mem::ManuallyDrop, rc::Rc, cell::RefCell,
};

// Texel that stores the SwapChain image format
pub type SwapchainFormat = BGRA<Normalized<u8>>;
pub type ForwardRendererRenderPass = RenderPass<SwapchainFormat, ()>;

// Main resource that will contain data to render objects on the screen
// This will contain the current swapchain texture that we must render to
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: ForwardRendererRenderPass,

    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynamicPipeline>>,
}

impl ForwardRenderer {
    // Create a new scene renderer
    pub fn new(
        render_pass: ForwardRendererRenderPass,
    ) -> Self {
        Self {
            render_pass,
            pipelines: Default::default(),
        }
    }

    // Register a new material pipeline within the renderer
    pub fn register<M: Material>(
        &mut self,
        graphics: &Graphics,
        assets: &Assets,
    ) -> MaterialId<M> {
        // Initialize the pipeline and register it if needed
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            log::debug!("Creating pipeline for material {}...", std::any::type_name::<M>());
            let pipeline = Pipeline::<M>::new(
                graphics,
                assets,
                &self.render_pass,
            );
            self.pipelines.insert(key, Rc::new(pipeline));
            log::debug!("Registered pipeline for material {}", std::any::type_name::<M>());
        }

        // Material ID is just a marker type for safety
        MaterialId(PhantomData)
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
