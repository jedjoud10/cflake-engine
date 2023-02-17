use crate::{DynamicPipeline, Material, MaterialId, Pipeline, CameraUniform, TimingUniform, SceneUniform};
use ahash::AHashMap;
use assets::Assets;
use bytemuck::Zeroable;
use graphics::{Graphics, Normalized, RenderPass, Texture2D, BGRA, PipelineInitializationError, UniformBuffer, BufferMode, BufferUsage, GpuPod, RGBA, SwapchainFormat};
use std::{
    any::TypeId, marker::PhantomData, mem::ManuallyDrop, rc::Rc, cell::RefCell,
};

// Renderpass that will render the scene
pub type ForwardRendererRenderPass = RenderPass<SwapchainFormat, ()>;
// Main resource that will contain data to render objects on the screen
// This will contain the current swapchain texture that we must render to
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: ForwardRendererRenderPass,

    // Data that will be sent to the shaders
    camera_buffer: UniformBuffer<CameraUniform>,
    timing_buffer: UniformBuffer<TimingUniform>,
    scene_buffer: UniformBuffer<SceneUniform>,

    // Material pipelines that we will use to render the surfaces
    pipelines: AHashMap<TypeId, Rc<dyn DynamicPipeline>>,
}

// Create a new uniform buffer with the given value (defaulted contents) 
fn create_uniform_buffer<T: GpuPod + Default>(graphics: &Graphics) -> UniformBuffer<T> {
    UniformBuffer::from_slice(graphics, &[T::default()], BufferMode::Dynamic, BufferUsage::default()).unwrap()
}

impl ForwardRenderer {
    // Create a new scene renderer
    pub(crate) fn new(
        graphics: &Graphics,
        render_pass: ForwardRendererRenderPass,
    ) -> Self {
        Self {
            render_pass,
            pipelines: Default::default(),
            camera_buffer: create_uniform_buffer::<CameraUniform>(graphics),
            timing_buffer: create_uniform_buffer::<TimingUniform>(graphics),
            scene_buffer: create_uniform_buffer::<SceneUniform>(graphics),
        }
    }

    // Register a new material pipeline within the renderer
    pub fn register<M: Material>(
        &mut self,
        graphics: &Graphics,
        assets: &Assets,
    ) -> Result<MaterialId<M>, PipelineInitializationError> {
        // Initialize the pipeline and register it if needed
        let key = TypeId::of::<M>();
        if !self.pipelines.contains_key(&key) {
            log::debug!("Creating pipeline for material {}...", utils::pretty_type_name::<M>());
            let pipeline = Pipeline::<M>::new(
                graphics,
                assets,
            )?;
            self.pipelines.insert(key, Rc::new(pipeline));
            log::debug!("Registered pipeline for material {}", utils::pretty_type_name::<M>());
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
