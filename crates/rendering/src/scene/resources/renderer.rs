use crate::{
    AlbedoMap, CameraBuffer, MaskMap, Mesh, NormalMap, SceneBuffer, TimingBuffer, WindowBuffer,
};

use assets::Assets;

use ecs::Entity;
use graphics::{
    ActiveRenderPass, ActiveRenderPipeline, BufferMode, BufferUsage, Depth, GpuPod,
    Graphics, LoadOp, Operation, RenderPass, SamplerFilter,
    SamplerMipMaps, SamplerSettings, SamplerWrap, StoreOp, Texel, Texture, Texture2D,
    TextureMipMaps, TextureMode, TextureUsage, UniformBuffer, RGBA, BGRA, SwapchainFormat, RenderPipeline, VertexModule, FragmentModule, Compiler, Shader, VertexConfig, PrimitiveConfig,
};
use utils::{Handle, Storage};

// Renderpass that will render the scene
pub type SceneColorLayout = (RGBA<f32>, RGBA<f32>, RGBA<f32>);
pub type SceneDepthLayout = Depth<f32>;

// Keeps tracks of data that we use for rendering the scene
// This will contain the G-Buffer and Depth Texture that we will use for deferred lighting 
pub struct DeferredRenderer {
    // Main deferred render pass that we will use to render to the swapchain
    pub(crate) deferred_render_pass: RenderPass<SceneColorLayout, SceneDepthLayout>,

    // G-Buffer and Depth Texture
    pub(crate) gbuffer_albedo_texture: Texture2D<RGBA<f32>>,
    pub(crate) gbuffer_normal_texture: Texture2D<RGBA<f32>>,
    pub(crate) gbuffer_mask_texture: Texture2D<RGBA<f32>>,
    pub(crate) depth_texture: Texture2D<SceneDepthLayout>,

    // Contains shader and render pass that will execute the lighting pass
    pub(crate) lighting_render_pass: RenderPass<SwapchainFormat, ()>,
    pub(crate) lighting_pipeline: RenderPipeline<SwapchainFormat, ()>,

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

// Create a new uniform buffer with default contents
fn create_uniform_buffer<T: GpuPod + Default>(graphics: &Graphics) -> UniformBuffer<T> {
    UniformBuffer::from_slice(
        graphics,
        &[T::default()],
        BufferMode::Dynamic,
        BufferUsage::WRITE | BufferUsage::READ,
    )
    .unwrap()
}

// Create a 4x4 texture 2D with the given value
fn create_texture2d<T: Texel>(graphics: &Graphics, value: T::Storage) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        Some(&[value; 16]),
        vek::Extent2::broadcast(4),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Create a texture that we will use for the G-Buffer
fn create_gbuffer_texture<T: Texel>(graphics: &Graphics, extent: vek::Extent2<u32>) -> Texture2D<T> {
    Texture2D::<T>::from_texels(
        graphics,
        None,
        extent,
        TextureMode::Resizable,
        TextureUsage::TARGET | TextureUsage::SAMPLED,
        Some(SamplerSettings {
            filter: SamplerFilter::Linear,
            wrap: SamplerWrap::Repeat,
            mipmaps: SamplerMipMaps::Auto,
        }),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}


// Load a engine default mesh
fn load_mesh(
    path: &str,
    assets: &Assets,
    graphics: &Graphics,
    storage: &mut Storage<Mesh>,
) -> Handle<Mesh> {
    let mesh = assets.load::<Mesh>((path, graphics.clone())).unwrap();
    storage.insert(mesh)
}

// Load the deferred shader
fn load_lighting_shader(assets: &Assets, graphics: &Graphics) -> Shader {
    // Load the vertex module for the deferred shader
    let vertex = assets
        .load::<VertexModule>("engine/shaders/common/quad.vert")
        .unwrap();

    // Load the fragment module for the deferred shader
    let fragment = assets
        .load::<FragmentModule>("engine/shaders/deferred/lighting.frag")
        .unwrap();

    // Create the bind layout for the compositor shader
    let mut compiler = Compiler::new(assets, graphics);
    Shader::new(vertex, fragment, &compiler).unwrap()
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
        let gbuffer_albedo_texture = create_gbuffer_texture::<RGBA<f32>>(graphics, extent);
        let gbuffer_normal_texture = create_gbuffer_texture::<RGBA<f32>>(graphics, extent);
        let gbuffer_mask_texture = create_gbuffer_texture::<RGBA<f32>>(graphics, extent);
        let depth_texture = create_gbuffer_texture::<Depth<f32>>(graphics, extent);

        // Tuple that contains the clear operations of the G-Buffer textures
        let color_operations = (
            Operation::<RGBA<f32>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<f32>> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            Operation::<RGBA<f32>> {
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

        // Load deferred shader and deferred render pass
        let lighting = load_lighting_shader(assets, graphics);
        let lighting_render_pass = RenderPass::<SwapchainFormat, ()>::new(
            graphics,
            Operation::<SwapchainFormat> {
                load: LoadOp::Clear(vek::Vec4::zero()),
                store: StoreOp::Store,
            },
            (),
        );

        // Create the display graphics pipeline
        let lighting_pipeline = RenderPipeline::<SwapchainFormat, ()>::new(
            graphics,
            None,
            None,
            None,
            VertexConfig::default(),
            PrimitiveConfig::Triangles {
                winding_order: graphics::WindingOrder::Ccw,
                cull_face: None,
                wireframe: false,
            },
            &lighting,
        )
        .unwrap();

        Self {
            // Render pass, G-Buffer textures, and depth texture
            deferred_render_pass: render_pass,
            gbuffer_albedo_texture,
            gbuffer_normal_texture,
            gbuffer_mask_texture,
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

            // Actual shading shader
            lighting_render_pass,
            lighting_pipeline,

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