use assets::Assets;
use ecs::Entity;
use math::Location;
use world::{Write, Storage};

use crate::{
    buffer::{UniformBuffer, ComputeStorage, BufferMode},
    mesh::Mesh,
    painter::Painter,
    prelude::{Depth, Ranged, Shader, Texture2D, RGB, SRGB, Sampling, Filter, Wrap, MipMapSetting, Texture, TextureMode}, context::{Window, Context}, material::{Sky, Standard}, display::Display, shader::{VertexStage, FragmentStage, ShaderCompiler, Processor},
};

use super::PointLight;

// Clustered shading is a method to render multiple lights
// efficienty without losing image quality
// The principle of "Clustered Shading" is to subdivide the camera's view frustum
// into multiple sub-regions called "clusters", and have the lights within them rendered
// TODO: Actually implement this lul
pub struct ClusteredShading {
    pub(crate) main_camera: Option<Entity>,
    pub(crate) skysphere_entity: Option<Entity>,
    pub(crate) painter: Painter<RGB<f32>, Depth<Ranged<u32>>, ()>,
    pub(crate) color_tex: Texture2D<RGB<f32>>,
    pub(crate) depth_tex: Texture2D<Depth<Ranged<u32>>>,
    pub(crate) main_directional_light: Option<Entity>,
    pub(crate) point_lights: ComputeStorage<(PointLight, Location)>,
    pub(crate) clusters: ComputeStorage<(u32, u32)>,
}

impl ClusteredShading {
    pub(crate) fn new(ctx: &mut Context, window: &Window, shaders: &mut Storage<Shader>, assets: &mut Assets) -> Self {
        // Settings for framebuffer textures
        let sampling = Sampling {
            filter: Filter::Nearest,
            wrap: Wrap::ClampToEdge,
        };
        let mipmaps = MipMapSetting::Disabled;
        
        // Create the color render texture
        let color = <Texture2D<RGB<f32>> as Texture>::new(
            ctx,
            TextureMode::Resizable,
            window.size(),
            sampling,
            mipmaps,
            None,
        )
        .unwrap();
    
        // Create the depth render texture
        let depth = <Texture2D<Depth<Ranged<u32>>> as Texture>::new(
            ctx,
            TextureMode::Resizable,
            window.size(),
            sampling,
            mipmaps,
            None,
        )
        .unwrap();
    
        // Create the default pipelines
        ctx.register_material::<Standard>(shaders, assets);
        ctx.register_material::<Sky>(shaders, assets);
    
        // Create the clustered shading rendererer
        let clustered_shading = ClusteredShading {
            main_camera: None,
            skysphere_entity: None,
            painter: Painter::new(ctx),
            color_tex: color,
            depth_tex: depth,
            main_directional_light: None,
            point_lights: ComputeStorage::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
            clusters: ComputeStorage::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
        };
        clustered_shading
    }

    // Get the main camera entity
    pub fn main_camera(&self) -> Option<Entity> {
        self.main_camera
    }

    // Get the main sky entity
    pub fn sky_entity(&self) -> Option<Entity> {
        self.skysphere_entity
    }

    // Get the main directional light entity
    pub fn main_directional_light(&self) -> Option<Entity> {
        self.main_directional_light
    }
}

// Directional shadow mapping for the main sun light
// The shadows must be rendered before we render the main frame
pub struct ShadowMapping {
    pub(crate) painter: Painter<(), Depth<Ranged<u32>>, ()>,
    pub(crate) depth_tex: Texture2D<Depth<Ranged<u32>>>,
    pub(crate) shader: Shader,
    pub(crate) resolution: u16,
}

impl ShadowMapping {
    pub(crate) fn new(resolution: u16, ctx: &mut Context, shaders: &mut Storage<Shader>, assets: &mut Assets) -> Self {
        // Settings for framebuffer textures
        let sampling = Sampling {
            filter: Filter::Nearest,
            wrap: Wrap::ClampToEdge,
        };
        let mipmaps = MipMapSetting::Disabled;

        // Create the depth render texture
        let depth_tex = <Texture2D<Depth<Ranged<u32>>> as Texture>::new(
            ctx,
            TextureMode::Resizable,
            vek::Extent2::broadcast(resolution),
            sampling,
            mipmaps,
            None,
        ).unwrap();

        // Load the shader used for shadow map object rasterization
        let vertex = assets.load::<VertexStage>("engine/shaders/shadow.vrsh.glsl").unwrap();
        let fragment = assets.load::<FragmentStage>("engine/shaders/shadow.frsh.glsl").unwrap();
        let shader = ShaderCompiler::link((vertex, fragment), Processor::new(assets), ctx);

        Self {
            painter: Painter::new(ctx),
            depth_tex,
            shader,
            resolution,
        }
    }
}

// This is a collection of post-processing effects that will
// be rendered onto the screen after we render the basic scene
pub struct PostProcessing {
    pub tonemapping_strength: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub vignette_strength: f32,
    pub vignette_size: f32,
    pub bloom_radius: f32,
    pub bloom_strength: f32,
    pub bloom_contrast: f32,
}

// The compositor is what we shall use to combine the clustered shading canvas and other composites
pub(crate) struct Compositor {
    pub(crate) quad: Mesh,
    pub(crate) compositor: Shader,
}

// These settings keep track what we rendered within a single frame
#[derive(Default, Debug, Clone, Copy)]
pub struct RenderedFrameStats {
    pub tris: u32,
    pub verts: u32,
    pub unique_materials: u32,
    pub unique_materials_shadow_casters: u32,
    pub material_instances: u32,
    pub rendered_surfaces: u32,
}
