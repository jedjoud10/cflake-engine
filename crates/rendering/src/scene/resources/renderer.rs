use std::{any::TypeId, rc::Rc, marker::PhantomData, mem::ManuallyDrop};
use ahash::AHashMap;
use assets::Assets;
use graphics::{RenderPass, Texture2D, BGRA, Normalized};
use crate::{DynamicPipeline, Material, MaterialId, Pipeline};

// Main resource that will contain data to render objects on the screen
// This will contain the current swapchain texture that we must render to
type SwapchainRenderTexture = Texture2D<BGRA<Normalized<u8>>>;
pub struct ForwardRenderer {    
    // Current render texture from the swapchain
    render_targets: Vec<ManuallyDrop<SwapchainRenderTexture>>,

    // Main render pass that we will use to render to the swapchain
    renderpass: RenderPass,

    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynamicPipeline>>
}

impl ForwardRenderer {
    // Create a new scene renderer
    pub fn new(render_targets: Vec<ManuallyDrop<SwapchainRenderTexture>>, renderpass: RenderPass) -> Self {
        Self {
            renderpass,
            render_targets,
            pipelines: Default::default(),
        }
    }

    // Register a new material pipeline within the renderer
    pub fn register<M: Material>(
        &mut self,
        assets: &Assets,
    ) -> MaterialId<M> {
        // Initialize the pipeline and register it if needed
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            let pipeline = Pipeline::<M>::new(assets, self.renderpass);
            self.pipelines.insert(key, Rc::new(pipeline));
        }

        // Material ID is just a marker type for safety
        MaterialId(PhantomData)
    }

    // Get a MaterialID from a pre-initialized pipeline
    pub fn get<M: Material>(&self) -> Option<MaterialId<M>> {
        let key = TypeId::of::<M>();
        self.pipelines
            .get(&key)
            .map(|_| MaterialId(PhantomData))
    }

    // Extract the internally stored material pipelines
    pub(crate) fn extract_pipelines(&self) -> Vec<Rc<dyn DynamicPipeline>> {
        self.pipelines
            .iter()
            .map(|(_key, value)| value.clone())
            .collect::<_>()
    }
}