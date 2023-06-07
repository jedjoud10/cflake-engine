use crate::{
    AlbedoMap, CameraBuffer, MaskMap, Mesh, NormalMap, SceneBuffer, TimingBuffer, WindowBuffer, create_texture2d, create_uniform_buffer,
};

use assets::Assets;

use ecs::Entity;
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, Depth, GpuPod,
    Graphics, LoadOp, Operation, RenderPass, SamplerFilter,
    SamplerMipMaps, SamplerSettings, SamplerWrap, StoreOp, Texel, Texture, Texture2D,
    TextureMipMaps, TextureMode, TextureUsage, UniformBuffer, RGBA, BGRA, SwapchainFormat, RenderPipeline, VertexModule, FragmentModule, Compiler, Shader, VertexConfig, PrimitiveConfig, Normalized, SamplerBorderColor,
};
use utils::{Handle, Storage};

// Renderpass that will render the scene
pub type SceneColorLayout = (RGBA<f32>, RGBA<Normalized<u8>>, RGBA<Normalized<i16>>, RGBA<Normalized<u8>>);
pub type SceneDepthLayout = Depth<f32>;

// Create a texture that we will use for the G-Buffer
pub(crate) fn create_gbuffer_texture<T: Texel>(graphics: &Graphics, extent: vek::Extent2<u32>) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        None,
        extent,
        TextureMode::Resizable,
        TextureUsage::TARGET | TextureUsage::SAMPLED,
        Some(SamplerSettings {
            mipmaps: SamplerMipMaps::Auto,
            comparison: None,
            mag_filter: SamplerFilter::Linear,
            min_filter: SamplerFilter::Linear,
            mip_filter: SamplerFilter::Linear,
            wraps_u: SamplerWrap::Repeat,
            wraps_v: SamplerWrap::Repeat,
            wraps_w: SamplerWrap::Repeat,
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
    pub(crate) gbuffer_position_texture: Texture2D<RGBA<f32>>,
    pub(crate) gbuffer_albedo_texture: Texture2D<RGBA<Normalized<u8>>>,
    pub(crate) gbuffer_normal_texture: Texture2D<RGBA<Normalized<i16>>>,
    pub(crate) gbuffer_mask_texture: Texture2D<RGBA<Normalized<u8>>>,
    pub(crate) depth_texture: Texture2D<SceneDepthLayout>,

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

    // Stats about shit drawn this frame
    pub drawn_unique_material_count: u32,
    pub material_instances_count: u32,
    pub rendered_direct_vertices_drawn: u64,
    pub rendered_direct_triangles_drawn: u64,
    pub culled_sub_surfaces: u64,
    pub rendered_sub_surfaces: u64,
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
        let gbuffer_position_texture = create_gbuffer_texture::<RGBA<f32>>(graphics, extent);
        let gbuffer_albedo_texture = create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, extent);
        let gbuffer_normal_texture = create_gbuffer_texture::<RGBA<Normalized<i16>>>(graphics, extent);
        let gbuffer_mask_texture = create_gbuffer_texture::<RGBA<Normalized<u8>>>(graphics, extent);
        let depth_texture = create_gbuffer_texture::<Depth<f32>>(graphics, extent);

        // Tuple that contains the clear operations of the G-Buffer textures
        let color_operations = (
            Operation::<RGBA<f32>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<Normalized<u8>>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<Normalized<i16>>> {
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
            gbuffer_position_texture,
            
            gbuffer_albedo_texture,
            gbuffer_normal_texture,
            gbuffer_mask_texture,
            depth_texture,

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
            drawn_unique_material_count: 0,
            material_instances_count: 0,
            rendered_direct_vertices_drawn: 0,
            rendered_direct_triangles_drawn: 0,
            culled_sub_surfaces: 0,
            rendered_sub_surfaces: 0,

            // Load the default meshes
            cube,
            icosphere,
            plane,
            sphere,
        }
    }
}