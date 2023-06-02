use assets::Assets;
use bytemuck::{Pod, Zeroable};
use graphics::{
    BufferMode, BufferUsage, Compiler, FragmentModule, Graphics, LoadOp, Operation,
    PrimitiveConfig, RenderPass, RenderPipeline, Shader, StoreOp, SwapchainFormat, Texture2D,
    UniformBuffer, VertexConfig, VertexModule, RGBA, Normalized, R, Depth,
};

use crate::{CameraUniform, SceneColorLayout, WindowUniform, SceneUniform};

// This is what will write to the swapchain
pub type FinalRenderPass = RenderPass<SwapchainFormat, ()>;
pub type FinalRenderPipeline = RenderPipeline<SwapchainFormat, ()>;

// Load the deferred lighting + post-processing shader
fn load_lighting_shader(assets: &Assets, graphics: &Graphics) -> Shader {
    // Load the vertex module for the deferred shader
    let vertex = assets
        .load::<VertexModule>("engine/shaders/common/quad.vert")
        .unwrap();

    // Load the fragment module for the deferred shader
    let fragment = assets
        .load::<FragmentModule>("engine/shaders/post/lighting.frag")
        .unwrap();

    // Create the bind layout for the compositor shader
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_sampled_texture::<Texture2D<RGBA<f32>>>("gbuffer_position_map");
    compiler.use_sampled_texture::<Texture2D<RGBA<Normalized<u8>>>>("gbuffer_albedo_map");
    compiler.use_sampled_texture::<Texture2D<RGBA<Normalized<i8>>>>("gbuffer_normal_map");
    compiler.use_sampled_texture::<Texture2D<RGBA<Normalized<u8>>>>("gbuffer_mask_map");
    compiler.use_sampled_texture::<Texture2D<Depth<f32>>>("depth_map");

    compiler.use_uniform_buffer::<CameraUniform>("camera");
    compiler.use_uniform_buffer::<SceneUniform>("scene");
    compiler.use_uniform_buffer::<PostProcessUniform>("post_processing");
    compiler.use_uniform_buffer::<WindowUniform>("window");
    
    Shader::new(vertex, fragment, &compiler).unwrap()
}


fn load_lighting_pass(graphics: &Graphics) -> RenderPass<graphics::BGRA<graphics::Normalized<u8>>, ()> {
    let render_pass = FinalRenderPass::new(
        graphics,
        Operation {
            load: LoadOp::Clear(vek::Vec4::broadcast(0)),
            store: StoreOp::Store,
        },
        (),
    );
    render_pass
}

fn load_lighting_pipeline(graphics: &Graphics, shader: Shader) -> RenderPipeline<graphics::BGRA<graphics::Normalized<u8>>, ()> {
    let pipeline = FinalRenderPipeline::new(
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
        &shader,
    )
    .unwrap();
    pipeline
}


// What tonemapping filter we should use
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tonemapping {
    // Reinhard tonemapping
    Reinhard,

    // Reinhard variant from shadertoy made by user "Jodie"
    ReinhardJodie,

    // ACES filmic curve
    ACES,

    // Clamps the HDR color values to LDR
    Clamp,
}

// How we should debug G-Buffer data
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DebugGBuffer {
    // World Space position
    Position,

    // Albedo Color
    Albedo,

    // World Space Normals 
    Normal,

    // Mask map (ao, roughness, metallic)
    Mask,

    // Disabled G-Buffer
    None,
}

impl Tonemapping {
    // Get a tonemap enum variant from raw discriminant index
    pub fn from_index(disc: u32) -> Self {
        match disc {
            0 => Self::Reinhard,
            1 => Self::ReinhardJodie,
            2 => Self::ACES,
            3 => Self::Clamp,
            _ => panic!(),
        }
    }

    // Get a tonemap discriminant index from enum variant
    pub fn into_index(&self) -> u32 {
        match self {
            Self::Reinhard => 0,
            Self::ReinhardJodie => 1,
            Self::ACES => 2,
            Self::Clamp => 3,
        }
    }
}

impl DebugGBuffer {
    // Get a debug g-buffer enum variant from raw discriminant index
    pub fn from_index(disc: u32) -> Self {
        match disc {
            0 => Self::Position,
            1 => Self::Albedo,
            2 => Self::Normal,
            3 => Self::Mask,
            u32::MAX => Self::None,
            _ => panic!()
        }
    }

    // Get a tonemap discriminant index from enum variant
    pub fn into_index(&self) -> u32 {
        match self {
            Self::Position => 0,
            Self::Albedo => 1,
            Self::Normal => 2,
            Self::Mask => 3,
            Self::None => u32::MAX,
        }
    }
}

// Container for post-processing parameters
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PostProcessUniform {
    // Lighting parameters
    pub exposure: f32,
    pub gamma: f32,

    // Vignette parameters
    pub vignette_strength: f32,
    pub vignette_size: f32,

    // Tonemapping parameters
    pub tonemapping_mode: u32,
    pub tonemapping_strength: f32,

    // Debug G-Buffer data
    pub debug_gbuffer: u32,
}

impl Default for PostProcessUniform {
    fn default() -> Self {
        Self {
            exposure: 2.0,
            gamma: 2.2,
            vignette_strength: 0.4,
            vignette_size: 0.1,
            tonemapping_mode: 2,
            tonemapping_strength: 1.0,
            debug_gbuffer: u32::MAX
        }
    }
}

// Overlays post-processing effects and multiple layers
// This will also render out the final composed image to the window
pub struct Compositor {
    // Contains shader and render pass that will execute the lighting pass
    pub(crate) lighting_render_pass: RenderPass<SwapchainFormat, ()>,
    pub(crate) lighting_pipeline: RenderPipeline<SwapchainFormat, ()>,

    // Post processing settings and buffer
    pub post_process: PostProcessUniform,
    pub(crate) post_process_buffer: UniformBuffer<PostProcessUniform>,
}

impl Compositor {
    // Create a new compositor that will mix and match multiple screen textures
    pub(crate) fn new(graphics: &Graphics, assets: &mut Assets) -> Self {
        let shader = load_lighting_shader(assets, graphics);
        let lighting_render_pass = load_lighting_pass(graphics);
        let lighting_pipeline = load_lighting_pipeline(graphics, shader);

        // Create a uniform buffer that will contain post processing parameters
        let post_process_buffer = UniformBuffer::from_slice(
            graphics,
            &[PostProcessUniform::default()],
            BufferMode::Dynamic,
            BufferUsage::WRITE,
        )
        .unwrap();

        Self {
            post_process: PostProcessUniform::default(),
            post_process_buffer,
            lighting_render_pass,
            lighting_pipeline,
        }
    }
}
