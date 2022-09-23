use assets::Assets;
use ecs::Entity;
use math::Location;
use vek::FrustumPlanes;
use world::{Storage};

use crate::{
    buffer::{ShaderBuffer, BufferMode},
    mesh::Mesh,
    painter::Painter,
    prelude::{Depth, Ranged, Shader, Texture2D, RGB, Sampling, Filter, Wrap, MipMapSetting, Texture, TextureMode}, context::{Window, Context}, material::{Sky, Standard}, display::Display, shader::{VertexStage, FragmentStage, ShaderCompiler, Processor},
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
    pub(crate) point_lights: ShaderBuffer<(PointLight, Location)>,
    pub(crate) clusters: ShaderBuffer<(u32, u32)>,
    pub(crate) cluster_size: u32,
}

impl ClusteredShading {
    pub(crate) fn new(ctx: &mut Context, cluster_size: u32, window: &Window, shaders: &mut Storage<Shader>, assets: &mut Assets) -> Self {
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
        
        ClusteredShading {
            main_camera: None,
            skysphere_entity: None,
            painter: Painter::new(ctx),
            color_tex: color,
            depth_tex: depth,
            main_directional_light: None,
            cluster_size,
            point_lights: ShaderBuffer::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
            clusters: ShaderBuffer::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
        }
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
    pub(crate) view_matrix: vek::Mat4<f32>,
    pub(crate) proj_matrix: vek::Mat4<f32>,
    pub(crate) shader: Shader,
}

impl ShadowMapping {
    pub(crate) fn new(size: f32, depth: f32, resolution: u16, ctx: &mut Context, _shaders: &mut Storage<Shader>, assets: &mut Assets) -> Self {
        // Settings for framebuffer textures
        let sampling = Sampling {
            filter: Filter::Nearest,
            wrap: Wrap::ClampToBorder(vek::Rgba::broadcast(1.0)),
        };
        let mipmaps = MipMapSetting::Disabled;

        // Create the depth render texture
        let depth_tex = <Texture2D<Depth<Ranged<u32>>> as Texture>::new(
            ctx,
            TextureMode::Dynamic,
            vek::Extent2::broadcast(resolution),
            sampling,
            mipmaps,
            None,
        ).unwrap();

        // Load the shader used for shadow map object rasterization
        let vertex = assets.load::<VertexStage>("engine/shaders/shadow.vrsh.glsl").unwrap();
        let fragment = assets.load::<FragmentStage>("engine/shaders/shadow.frsh.glsl").unwrap();
        let shader = ShaderCompiler::link((vertex, fragment), Processor::new(assets), ctx);

        // The shadow frustum is the cuboid that will contain the shadow map
        let frustum = FrustumPlanes::<f32> {
            left: -size,
            right: size,
            bottom: -size,
            top: size,
            near: -depth / 2.0,
            far: depth / 2.0,
        };

        // Create the projection matrix from the frustum
        let proj_matrix = vek::Mat4::orthographic_rh_no(frustum);

        Self {
            painter: Painter::new(ctx),
            depth_tex,
            shader,
            view_matrix: vek::Mat4::identity(),
            proj_matrix,
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
    pub shadow_casters_tris: u32,
    pub shadow_casters_verts: u32,
    pub shadow_casters_surfaces: u32,
}
