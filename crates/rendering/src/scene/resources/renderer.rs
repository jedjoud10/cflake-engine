use crate::{
    AlbedoMap, AlbedoTexel, CameraBuffer, CameraUniform,
    DynamicPipeline, Material, MaterialId, NormalMap, NormalTexel,
    Pipeline, SceneBuffer, SceneUniform, TimingBuffer, TimingUniform, WindowBuffer, WindowUniform,
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
    UniformBuffer, BGRA, RGBA, Depth, SamplerFilter, SamplerWrap, SamplerMipMaps, TextureImportSettings, ActiveRenderPass, ActiveGraphicsPipeline,
};
use std::{
    any::TypeId, cell::RefCell, marker::PhantomData,
    mem::ManuallyDrop, rc::Rc,
};

// Renderpass that will render the scene
pub type SceneColor = RGBA<Normalized<u8>>;
pub type SceneDepth = Depth<f32>;
pub type SceneRenderPass = RenderPass<SceneColor, SceneDepth>;
pub type ActiveSceneRenderPass<'r, 't> = ActiveRenderPass<'r, 't, SceneColor, SceneDepth>;
pub type ActiveScenePipeline<'a, 'r, 't> = ActiveGraphicsPipeline<'a, 'r, 't, SceneColor, SceneDepth>;

// Keeps tracks of data that we use for rendering the scene
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: SceneRenderPass,
    
    // Since we use post processing, we will write to a texture instead
    pub(crate) color_texture: Texture2D<SceneColor>,
    pub(crate) depth_texture: Texture2D<SceneDepth>,

    // Main camera entity that we use to render the scene
    pub main_camera: Option<Entity>,

    // Default shader buffers that will be shared with each material
    pub camera_buffer: CameraBuffer,
    pub timing_buffer: TimingBuffer,
    pub scene_buffer: SceneBuffer,
    pub window_buffer: WindowBuffer,

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
        // Create the render pass color texture
        let color_texture = Texture2D::<RGBA<Normalized<u8>>>::from_texels(
            graphics,
            None,
            extent,
            TextureMode::Resizable,
            TextureUsage::RENDER_TARGET | TextureUsage::SAMPLED,
            SamplerSettings {
                filter: SamplerFilter::Linear,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            },
            TextureMipMaps::Disabled
        ).unwrap();

        // Create the render pass depth texture
        let depth_texture = Texture2D::<Depth::<f32>>::from_texels(
            graphics,
            None,
            extent,
            TextureMode::Resizable,
            TextureUsage::RENDER_TARGET,
            SamplerSettings {
                filter: SamplerFilter::Linear,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            },
            TextureMipMaps::Disabled
        ).unwrap();

        // Create the forward shading scene pass
        let render_pass = SceneRenderPass::new(
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

        // Create the default 1x1 textures colors
        let white = vek::Vec3::broadcast(255).with_w(255);
        let black = vek::Vec4::broadcast(0);
        let normal = vek::Vec3::new(127, 127, 255).with_w(255); ;
            
        Self {
            // Render pass, color texture, and depth texture
            render_pass,
            color_texture,
            depth_texture,

            // Create the common material buffers
            camera_buffer: create_uniform_buffer(graphics),
            timing_buffer: create_uniform_buffer(graphics),
            scene_buffer: create_uniform_buffer(graphics),
            window_buffer: create_uniform_buffer(graphics),

            // Create the 1x1 common textures
            white: create_texture2d(graphics, normal),
            black: create_texture2d(graphics, normal),
            normal: create_texture2d(graphics, normal),

            // No default camera
            main_camera: None,
            sky_gradient,
        }
    }
}
