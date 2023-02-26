use crate::{
    AlbedoMap, AlbedoTexel, CameraBuffer, CameraUniform,
    DynamicPipeline, Material, MaterialId, NormalMap, NormalTexel,
    Pipeline, SceneBuffer, SceneUniform, TimingBuffer, TimingUniform,
};
use ahash::AHashMap;
use assets::Assets;
use bytemuck::Zeroable;
use ecs::Entity;
use graphics::{
    BufferMode, BufferUsage, GpuPod, Graphics, LoadOp, Normalized,
    Operation, PipelineInitializationError, RenderPass,
    SamplerSettings, StoreOp, SwapchainFormat, Texel, Texture,
    Texture2D, TextureMipMaps, TextureMode, TextureUsage,
    UniformBuffer, BGRA, RGBA, Depth, SamplerFilter, SamplerWrap, SamplerMipMaps, TextureImportSettings,
};
use std::{
    any::TypeId, cell::RefCell, marker::PhantomData,
    mem::ManuallyDrop, rc::Rc,
};

// Renderpass that will render the scene
pub type ForwardRendererRenderPass = RenderPass<SwapchainFormat, Depth<f32>>;

// Keeps tracks of data that we use for rendering the scene
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: ForwardRendererRenderPass,
    pub(crate) depth_texture: Texture2D<Depth<f32>>,

    // Main camera entity that we use to render the scene
    pub main_camera: Option<Entity>,

    // Default shader buffers that will be shared with each material
    pub camera_buffer: CameraBuffer,
    pub timing_buffer: TimingBuffer,
    pub scene_buffer: SceneBuffer,

    // Default textures that will be shared with each material
    pub white: AlbedoMap,
    pub black: AlbedoMap,
    pub normal: NormalMap,

    // Default sky gradient texture
    pub sky_gradient: AlbedoMap,
}

// Create a new uniform buffer with default contents
fn create_uniform_buffer<T: GpuPod + Default>(
    graphics: &Graphics,
) -> UniformBuffer<T> {
    UniformBuffer::from_slice(
        graphics,
        &[T::default()],
        BufferMode::Dynamic,
        BufferUsage::WRITE,
    )
    .unwrap()
}

// Create a 1x1 texture 2D with the given value
fn create_texture2d<T: Texel>(
    graphics: &Graphics,
    value: T::Storage,
) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        Some(&[value]),
        vek::Extent2::broadcast(1),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        SamplerSettings::default(),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

impl ForwardRenderer {
    // Create a new scene render pass and the forward renderer
    pub(crate) fn new(graphics: &Graphics, assets: &mut Assets, extent: vek::Extent2<u32>) -> Self {
        // Create the depth texture for the forward shading scene pass
        let depth_texture = Texture2D::<Depth::<f32>>::from_texels(
            graphics,
            None,
            extent,
            TextureMode::Dynamic,
            TextureUsage::RENDER_TARGET,
            SamplerSettings {
                filter: SamplerFilter::Linear,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            },
            TextureMipMaps::Disabled
        ).unwrap();

        // Create the forward shading scene pass
        let render_pass = ForwardRendererRenderPass::new(
            &graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0)),
                store: StoreOp::Store,
            },
            Operation {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            },
        ).unwrap();

        // Load the default sky gradient texture
        let sky_gradient = assets.load::<AlbedoMap>(
            ("engine/textures/scene/sky.jpg",
            graphics.clone(),
            TextureImportSettings {
                mipmaps: TextureMipMaps::Disabled,
                ..Default::default()
            }
        )).unwrap();

        Self {
            // Render pass and it's depth texture
            render_pass,
            depth_texture,

            // Create the common material buffers
            camera_buffer: create_uniform_buffer::<CameraUniform>(
                graphics,
            ),
            timing_buffer: create_uniform_buffer::<TimingUniform>(
                graphics,
            ),
            scene_buffer: create_uniform_buffer::<SceneUniform>(
                graphics,
            ),

            // Create the 1x1 common textures
            white: create_texture2d::<AlbedoTexel>(
                graphics,
                vek::Vec3::broadcast(255).with_w(255),
            ),
            black: create_texture2d::<AlbedoTexel>(
                graphics,
                vek::Vec4::broadcast(0),
            ),
            normal: create_texture2d::<NormalTexel>(
                graphics,
                vek::Vec3::new(127, 127, 255).with_w(255),
            ),

            // No default camera
            main_camera: None,

            sky_gradient,
        }
    }
}
