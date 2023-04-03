use crate::{
    AlbedoMap, CameraBuffer, MaskMap, NormalMap, SceneBuffer,
    TimingBuffer, WindowBuffer,
};

use assets::Assets;

use ecs::Entity;
use graphics::{
    ActiveGraphicsPipeline, ActiveRenderPass, BufferMode,
    BufferUsage, Depth, GpuPod, Graphics, LoadOp, Operation,
    RenderPass, SamplerFilter, SamplerMipMaps, SamplerSettings,
    SamplerWrap, StoreOp, Texel, Texture, Texture2D, TextureMipMaps, TextureMode, TextureUsage,
    UniformBuffer, RGBA,
};
use utils::{Handle, Storage};

// Renderpass that will render the scene
pub type SceneColor = RGBA<f32>;
pub type SceneDepth = Depth<f32>;
pub type SceneRenderPass = RenderPass<SceneColor, SceneDepth>;
pub type ActiveSceneRenderPass<'r, 't> =
    ActiveRenderPass<'r, 't, SceneColor, SceneDepth>;
pub type ActiveScenePipeline<'a, 'r, 't> =
    ActiveGraphicsPipeline<'a, 'r, 't, SceneColor, SceneDepth>;

// Keeps tracks of data that we use for rendering the scene
pub struct ForwardRenderer {
    // Main render pass that we will use to render to the swapchain
    pub(crate) render_pass: SceneRenderPass,

    // Since we use post processing, we will write to a texture instead
    pub(crate) color_texture: Texture2D<SceneColor>,
    pub(crate) depth_texture: Texture2D<SceneDepth>,

    // Main camera entity that we use to render the scene
    pub main_camera: Option<Entity>,

    // Main directional light that will enlighten our world
    pub main_directional_light: Option<Entity>,

    // Default shader buffers that will be shared with each material
    pub camera_buffer: CameraBuffer,
    pub timing_buffer: TimingBuffer,
    pub scene_buffer: SceneBuffer,
    pub window_buffer: WindowBuffer,

    // Default textures that will be shared with each material
    pub white: Handle<AlbedoMap>,
    pub black: Handle<AlbedoMap>,
    pub normal: Handle<NormalMap>,
    pub mask: Handle<MaskMap>,
}

// Create a new uniform buffer with default contents
fn create_uniform_buffer<T: GpuPod + Default>(
    graphics: &Graphics,
) -> UniformBuffer<T> {
    UniformBuffer::from_slice(
        graphics,
        &[T::default()],
        BufferMode::Dynamic,
        BufferUsage::WRITE | BufferUsage::READ,
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
        Some(&[value; 16]),
        vek::Extent2::broadcast(4),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        SamplerSettings::default(),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

impl ForwardRenderer {
    // Create a new scene render pass and the forward renderer
    pub(crate) fn new(
        graphics: &Graphics,
        _assets: &Assets,
        extent: vek::Extent2<u32>,
        albedo_maps: &mut Storage<AlbedoMap>,
        normal_maps: &mut Storage<NormalMap>,
        mask_maps: &mut Storage<MaskMap>,
    ) -> Self {
        // Create the render pass color texture
        let color_texture = Texture2D::<RGBA<f32>>::from_texels(
            graphics,
            None,
            extent,
            TextureMode::Resizable,
            TextureUsage::TARGET | TextureUsage::SAMPLED,
            SamplerSettings {
                filter: SamplerFilter::Linear,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            },
            TextureMipMaps::Disabled,
        )
        .unwrap();

        // Create the render pass depth texture
        let depth_texture = Texture2D::<Depth<f32>>::from_texels(
            graphics,
            None,
            extent,
            TextureMode::Resizable,
            TextureUsage::TARGET | TextureUsage::SAMPLED,
            SamplerSettings {
                filter: SamplerFilter::Linear,
                wrap: SamplerWrap::Repeat,
                mipmaps: SamplerMipMaps::Auto,
            },
            TextureMipMaps::Disabled,
        )
        .unwrap();

        // Create the forward shading scene pass
        let render_pass = SceneRenderPass::new(
            graphics,
            Operation {
                load: LoadOp::Clear(vek::Vec4::broadcast(0f32)),
                store: StoreOp::Store,
            },
            Operation {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            },
        );

        // Create the default 1x1 textures colors
        let white = vek::Vec4::broadcast(255);
        let black = vek::Vec4::broadcast(0);
        let normal = vek::Vec2::new(127, 127);
        let mask = vek::Vec4::new(255u8, 255, 0, 0);

        // Create the 1x1 default textures
        let white =
            albedo_maps.insert(create_texture2d(graphics, white));
        let black =
            albedo_maps.insert(create_texture2d(graphics, black));
        let normal =
            normal_maps.insert(create_texture2d(graphics, normal));
        let mask = mask_maps.insert(create_texture2d(graphics, mask));

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

            // Use the handles of the default textures
            white,
            black,
            normal,
            mask,

            // No default camera
            main_camera: None,
            main_directional_light: None,
        }
    }
}
