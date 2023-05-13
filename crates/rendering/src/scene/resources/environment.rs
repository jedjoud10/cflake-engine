use assets::Assets;
use graphics::{Graphics, RenderPass, RGBA, RenderPipeline, Shader, CubeMap, Texel, ImageTexel, TextureMode, TextureUsage, TextureMipMaps, SamplerSettings, Texture};

pub type EnvironmentMap = CubeMap<RGBA<f32>>;


// Create a cubemap with a specific resolution
fn create_cubemap<T: Texel + ImageTexel>(graphics: &Graphics, value: T::Storage, resolution: usize) -> CubeMap<T> {
    CubeMap::<T>::from_texels(
        graphics,
        Some(&vec![value; resolution*resolution*6]),
        vek::Extent2::broadcast(resolution as u32),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::TARGET | TextureUsage::COPY_DST,
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Environment maps that contains the diffuse, specular, and ambient cubemaps
// This also contains some settings on how we should create the procedural environment sky
pub struct Environment {
    // Double buffered environment map
    pub environment_map: [EnvironmentMap; 2],

    // Render pass and shader to write to the environment map
    /*
    pub(crate) environment_map_render_pass: RenderPass<RGBA<f32>, ()>,
    pub(crate) environment_map_shader: Shader,
    pub(crate) environment_map_render_pipeline: RenderPipeline<RGBA<f32>, ()>,
    */
}

impl Environment {
    // Create a new scene environment render passes and cubemaps
    pub(crate) fn new(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Self {
        Self {
            environment_map: [
                create_cubemap(graphics, vek::Vec4::zero(), 128),
                create_cubemap(graphics, vek::Vec4::zero(), 128)
            ],
        }
    }
}