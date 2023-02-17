use crate::{DynamicPipeline, Material, MaterialId, Pipeline, CameraUniform, TimingUniform, SceneUniform, AlbedoMap, NormalMap, CameraBuffer, TimingBuffer, SceneBuffer, NormalTexel, AlbedoTexel};
use ahash::AHashMap;
use assets::Assets;
use bytemuck::Zeroable;
use graphics::{Graphics, Normalized, RenderPass, Texture2D, BGRA, PipelineInitializationError, UniformBuffer, BufferMode, BufferUsage, GpuPod, RGBA, SwapchainFormat, Operation, LoadOp, StoreOp, Texel, TextureMode, TextureUsage, SamplerSettings, TextureMipMaps, Texture};
use std::{
    any::TypeId, marker::PhantomData, mem::ManuallyDrop, rc::Rc, cell::RefCell,
};

// Renderpass that will render the scene
pub type ForwardRendererRenderPass = RenderPass<SwapchainFormat, ()>;

// Keeps tracks of data that we use for rendering the scene 
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: ForwardRendererRenderPass,

    // Default shader buffers that will be shared with each material
    pub camera_buffer: CameraBuffer,
    pub timing_buffer: TimingBuffer,
    pub scene_buffer: SceneBuffer,    

    // Default textures that will be shared with each material
    pub white: AlbedoMap,
    pub black: AlbedoMap,
    pub normal: NormalMap,
}

// Create a new uniform buffer with default contents
fn create_uniform_buffer<T: GpuPod + Default>(graphics: &Graphics) -> UniformBuffer<T> {
    UniformBuffer::from_slice(graphics, &[T::default()], BufferMode::Dynamic, BufferUsage::default()).unwrap()
}

// Create a 1x1 texture 2D with the given value
fn create_texture2d<T: Texel>(graphics: &Graphics, value: T::Storage) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        Some(&[value]),
        vek::Extent2::broadcast(1),
        TextureMode::Dynamic,
        TextureUsage::Placeholder,
        SamplerSettings::default(),
        TextureMipMaps::Disabled
    ).unwrap()
}

impl ForwardRenderer {
    // Create a new scene render pass and the forward renderer
    pub(crate) fn new(
        graphics: &Graphics,
    ) -> Self {
        // Create the forward shading scene pass
        let render_pass = ForwardRendererRenderPass::new(
            &graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0)),
                store: StoreOp::Store,
            },
            (),
        ).unwrap();

        Self {
            render_pass,

            // Create the common material buffers
            camera_buffer: create_uniform_buffer::<CameraUniform>(graphics),
            timing_buffer: create_uniform_buffer::<TimingUniform>(graphics),
            scene_buffer: create_uniform_buffer::<SceneUniform>(graphics),

            // Create the 1x1 common textures
            white: create_texture2d::<AlbedoTexel>(graphics, vek::Vec4::broadcast(0).with_w(255)),
            black: create_texture2d::<AlbedoTexel>(graphics, vek::Vec4::broadcast(255)),
            normal: create_texture2d::<NormalTexel>(graphics, vek::Vec4::new(0, 0, 127, 127)),
        }
    }
}