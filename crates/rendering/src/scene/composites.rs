use assets::Assets;
use ecs::Entity;
use math::Location;
use vek::FrustumPlanes;
use world::{Handle, Storage};

use crate::{
    buffer::{BufferMode, ShaderBuffer, UniformBuffer},
    context::{Context, Window},
    display::Display,
    material::{AlbedoMap, IntegrationMap, MaskMap, NormalMap, Sky, Standard},
    mesh::Mesh,
    others::Comparison,
    painter::Painter,
    prelude::{
        Depth, Filter, MipMapSetting, Ranged, Sampling, Shader, Texture, Texture2D,
        TextureImportSettings, TextureMode, Wrap, RG, RGB,
    },
    shader::{ComputeShader, FragmentStage, Processor, ShaderCompiler, VertexStage},
};

use super::{PackedPointLight, PointLight};

// This resource will contain the common shared textures that we use througouht the renderer
// This contains the default white, black, normal, and mask map textures
pub struct CommonTextures {
    pub white: AlbedoMap,
    pub black: AlbedoMap,
    pub normal: NormalMap,
    pub mask: MaskMap,
}

impl CommonTextures {
    pub(crate) fn new(ctx: &mut Context) -> Self {
        // Create the default white texture
        let white = AlbedoMap::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::default(),
            MipMapSetting::Disabled,
            Some(&[vek::Vec4::<u8>::one() * 255]),
        )
        .unwrap();

        // Create the default black texture
        let black = AlbedoMap::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::default(),
            MipMapSetting::Disabled,
            Some(&[vek::Vec4::<u8>::zero()]),
        )
        .unwrap();

        // Create the default normal texture
        let normal = NormalMap::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::default(),
            MipMapSetting::Disabled,
            Some(&[vek::Vec3::new(128, 128, 255)]),
        )
        .unwrap();

        // Create the default mask texture
        let mask = MaskMap::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::default(),
            MipMapSetting::Disabled,
            Some(&[vek::Vec4::new(255, 255, 255, 0)]),
        )
        .unwrap();

        Self {
            white,
            black,
            normal,
            mask,
        }
    }
}

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
    pub(crate) point_lights: ShaderBuffer<PackedPointLight>,
    pub(crate) point_light_ids: ShaderBuffer<u32>,
    pub(crate) clusters: ShaderBuffer<(u32, u32)>,
    pub(crate) brdf_integration_map: IntegrationMap,
    //pub(crate) compute: ComputeShader,
    pub(crate) cluster_size: u32,
}

impl ClusteredShading {
    pub(crate) fn new(
        ctx: &mut Context,
        cluster_size: u32,
        window: &Window,
        shaders: &mut Storage<Shader>,
        assets: &mut Assets,
    ) -> Self {
        // Settings for framebuffer textures
        let sampling = Sampling {
            filter: Filter::Nearest,
            wrap: Wrap::ClampToEdge,
            ..Default::default()
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

        // Load the BRDF integration map
        let brdf_integration_map = assets
            .load_with::<IntegrationMap>(
                "engine/textures/integration.png",
                (
                    ctx,
                    TextureImportSettings {
                        sampling: Sampling {
                            filter: Filter::Linear,
                            wrap: Wrap::ClampToEdge,
                            ..Default::default()
                        },
                        mode: TextureMode::Static,
                        mipmaps: MipMapSetting::Disabled,
                    },
                ),
            )
            .unwrap();

        // Create the default pipelines
        ctx.register_material::<Standard>(shaders, assets);
        ctx.register_material::<Sky>(shaders, assets);

        // TODO: Create the cluster compute shader that will sort the lights

        // Create the clustered shading rendererer
        ClusteredShading {
            main_camera: None,
            skysphere_entity: None,
            painter: Painter::new(ctx),
            color_tex: color,
            depth_tex: depth,
            main_directional_light: None,
            cluster_size,
            point_light_ids: ShaderBuffer::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
            point_lights: ShaderBuffer::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
            clusters: ShaderBuffer::from_slice(ctx, &[], BufferMode::Resizable).unwrap(),
            brdf_integration_map,
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
    pub(crate) fn new(
        size: f32,
        depth: f32,
        resolution: u16,
        ctx: &mut Context,
        _shaders: &mut Storage<Shader>,
        assets: &mut Assets,
    ) -> Self {
        let sampling = Sampling {
            filter: Filter::Linear,
            wrap: Wrap::ClampToBorder(vek::Rgba::broadcast(1.0f32)),
            depth_comparison: Some(Comparison::GreaterThanOrEquals),
            ..Default::default()
        };

        // Create the depth shadow map texture
        let depth_tex = <Texture2D<Depth<Ranged<u32>>> as Texture>::new(
            ctx,
            TextureMode::Dynamic,
            vek::Extent2::broadcast(resolution),
            sampling,
            MipMapSetting::Disabled,
            None,
        )
        .unwrap();

        // Load the shader used for shadow map object rasterization
        let vertex = assets
            .load::<VertexStage>("engine/shaders/projection.vrtx.glsl")
            .unwrap();
        let fragment = assets
            .load::<FragmentStage>("engine/shaders/depth.frag.glsl")
            .unwrap();
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

// How we will finally tonemap the rendered scene
#[derive(Default, Clone, Copy)]
#[repr(u8)]
pub enum ToneMappingMode {
    // This will use the aces filmic curve
    #[default]
    ACES,

    // This will use the reinhard tonemapping curve
    Reinhard,
}

// This is a collection of post-processing effects that will
// be rendered onto the screen after we render the basic scene
pub struct PostProcessing {
    pub tonemapping_strength: f32,
    pub tonemapper: ToneMappingMode,
    pub exposure: f32,
    pub gamma: f32,
    pub vignette_strength: f32,
    pub vignette_size: f32,
}

impl Default for PostProcessing {
    fn default() -> Self {
        Self {
            tonemapper: ToneMappingMode::ACES,
            tonemapping_strength: 1.0,
            exposure: 1.2,
            gamma: 2.2,
            vignette_strength: 0.0,
            vignette_size: 0.2,
        }
    }
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
