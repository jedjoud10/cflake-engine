use crate::{
    AlbedoMap, CameraBuffer, MaskMap, Mesh, NormalMap, SceneBuffer, TimingBuffer, WindowBuffer, create_texture2d, create_uniform_buffer, PassStats,
};

use assets::Assets;

use ecs::Entity;
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, Depth, GpuPod,
    Graphics, LoadOp, Operation, RenderPass, SamplerFilter,
    SamplerMipMaps, SamplerSettings, SamplerWrap, StoreOp, Texel, Texture, Texture2D,
    TextureMipMaps, TextureUsage, UniformBuffer, RGBA, BGRA, SwapchainFormat, RenderPipeline, VertexModule, FragmentModule, Compiler, Shader, VertexConfig, PrimitiveConfig, Normalized, SamplerBorderColor, TextureViewSettings, Region,
};
use utils::{Handle, Storage};

// Renderpass that will render the scene
pub type SceneColorLayout = (RGBA<Normalized<u8>>, RGBA<Normalized<i8>>, RGBA<Normalized<u8>>);
pub type SceneDepthLayout = Depth<f32>;

// Create a texture that we will use for the G-Buffer
pub(crate) fn create_gbuffer_texture<T: Texel>(graphics: &Graphics, extent: vek::Extent2<u32>) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        None,
        extent,
        TextureUsage::TARGET | TextureUsage::SAMPLED,
        &[TextureViewSettings::whole::<<Texture2D<T> as Texture>::Region>()],
        Some(SamplerSettings {
            mipmaps: SamplerMipMaps::Auto,
            comparison: None,
            mag_filter: SamplerFilter::Linear,
            min_filter: SamplerFilter::Linear,
            mip_filter: SamplerFilter::Linear,
            wrap_u: SamplerWrap::Repeat,
            wrap_v: SamplerWrap::Repeat,
            wrap_w: SamplerWrap::Repeat,
            border: SamplerBorderColor::OpaqueBlack,
        }),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Load a engine default mesh
pub(crate) fn load_mesh(
    path: &str,
    assets: &Assets,
    graphics: &Graphics,
    storage: &mut Storage<Mesh>,
) -> Handle<Mesh> {
    let mesh = assets.load::<Mesh>((path, graphics.clone())).unwrap();
    storage.insert(mesh)
}

// Keeps tracks of data that we use for rendering the scene
// This will contain the G-Buffer and Depth Texture that we will use for deferred lighting 
pub struct DeferredRenderer {
    // Main deferred render pass that we will use to render to the swapchain
    pub(crate) deferred_render_pass: RenderPass<SceneColorLayout, SceneDepthLayout>,

    // G-Buffer and Depth Texture
    pub(crate) gbuffer_albedo_texture: Texture2D<RGBA<Normalized<u8>>>,
    pub(crate) gbuffer_normal_texture: Texture2D<RGBA<Normalized<i8>>>,
    pub(crate) gbuffer_mask_texture: Texture2D<RGBA<Normalized<u8>>>,
    pub(crate) depth_texture: Texture2D<SceneDepthLayout>,
    pub(crate) window_size: vek::Extent2<u32>,

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

    // Load the common models
    pub cube: Handle<Mesh>,
    pub icosphere: Handle<Mesh>,
    pub plane: Handle<Mesh>,
    pub sphere: Handle<Mesh>,

    // Stats for the deferred and shadow pass
    pub deferred_pass_stats: PassStats,
    pub shadow_pass_stats: PassStats,
}

impl DeferredRenderer {
    // Create a new scene render pass and the forward renderer
    pub(crate) fn new(
        graphics: &Graphics,
        assets: &Assets,
        extent: vek::Extent2<u32>,
        meshes: &mut Storage<Mesh>,
        albedo_maps: &mut Storage<AlbedoMap>,
        normal_maps: &mut Storage<NormalMap>,
        mask_maps: &mut Storage<MaskMap>,
    ) -> Self {
        // Create the G-Buffer textures and depth texture
        let gbuffer_albedo_texture = create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, extent);
        let gbuffer_normal_texture = create_gbuffer_texture::<RGBA<Normalized<i8>>>(graphics, extent);
        let gbuffer_mask_texture = create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, extent);
        let depth_texture = create_gbuffer_texture::<Depth<f32>>(graphics, extent);

        // Tuple that contains the clear operations of the G-Buffer textures
        let color_operations = (
            Operation::<RGBA<Normalized<u8>>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<Normalized<i8>>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<Normalized<u8>>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
        );
        
        // Clear operation of the depth texture
        let depth_stencil_operations = Operation {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store,
        };

        // Create the deferred scene pass that will write to the G-Buffer
        let render_pass = RenderPass::<SceneColorLayout, SceneDepthLayout>::new(
            graphics,
            color_operations,
            depth_stencil_operations,
        );

        // Create the default 1x1 textures colors
        let white = vek::Vec4::broadcast(255);
        let black = vek::Vec4::broadcast(0);
        let normal = vek::Vec2::new(127, 127);
        let mask = vek::Vec4::new(255u8, 255, 255, 0);

        // Create the 1x1 default textures
        let white = albedo_maps.insert(create_texture2d(graphics, white));
        let black = albedo_maps.insert(create_texture2d(graphics, black));
        let normal = normal_maps.insert(create_texture2d(graphics, normal));
        let mask = mask_maps.insert(create_texture2d(graphics, mask));

        // Load the default meshes
        let cube = load_mesh("engine/meshes/cube.obj", assets, graphics, meshes);
        let icosphere = load_mesh("engine/meshes/icosphere.obj", assets, graphics, meshes);
        let plane = load_mesh("engine/meshes/plane.obj", assets, graphics, meshes);
        let sphere = load_mesh("engine/meshes/sphere.obj", assets, graphics, meshes);

        Self {
            // Render pass, G-Buffer textures, and depth texture
            deferred_render_pass: render_pass,
            
            gbuffer_albedo_texture,
            gbuffer_normal_texture,
            gbuffer_mask_texture,
            depth_texture,
            window_size: extent,

            // Create the common material buffers
            camera_buffer: create_uniform_buffer::<_, 1>(graphics, BufferUsage::WRITE),
            timing_buffer: create_uniform_buffer::<_, 1>(graphics, BufferUsage::WRITE),
            scene_buffer: create_uniform_buffer::<_, 1>(graphics, BufferUsage::WRITE),
            window_buffer: create_uniform_buffer::<_, 1>(graphics, BufferUsage::WRITE),

            // Use the handles of the default textures
            white,
            black,
            normal,
            mask,


            // No default camera
            main_camera: None,
            main_directional_light: None,

            // Statistics
            deferred_pass_stats: Default::default(),
            shadow_pass_stats: Default::default(),

            // Load the default meshes
            cube,
            icosphere,
            plane,
            sphere,
        }
    }
}