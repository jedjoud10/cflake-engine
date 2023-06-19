use std::mem::size_of;

use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputeShader, CubeMap, Graphics, ImageTexel,
    LayeredTexture2D, SamplerSettings,
    StorageAccess, Texel, Texture, TextureMipMaps, TextureUsage, RGBA, TextureViewSettings, TextureViewDimension, UniformBuffer, BufferMode, BufferUsage, ModuleVisibility,
};

pub type EnvironmentMap = CubeMap<RGBA<f32>>;

// Create a cubemap with a specific resolution
fn create_cubemap<T: Texel + ImageTexel>(
    graphics: &Graphics,
    resolution: u32,
) -> CubeMap<T> {
    CubeMap::<T>::from_texels(
        graphics,
        None,
        vek::Extent2::broadcast(resolution),
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
        &[
            TextureViewSettings::whole::<<CubeMap<T> as Texture>::Region>(),
            TextureViewSettings {
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
                dimension: TextureViewDimension::D2Array,
            }
        ],
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Environment maps that contains the diffuse, specular, and ambient cubemaps
// This also contains some settings on how we should create the procedural environment sky
pub struct Environment {
    pub(crate) environment_map: EnvironmentMap,
    pub(crate) diffuse_ibl_map: EnvironmentMap, 
    pub(crate) resolution: u32,
    pub(crate) environment_shader: ComputeShader,
    pub(crate) ibl_diffuse_convolution_shader: ComputeShader,
    pub(crate) matrices: UniformBuffer<vek::Vec4<vek::Vec4<f32>>>,
}

impl Environment {
    // Create a new scene environment render passes and cubemaps
    pub(crate) fn new(graphics: &Graphics, assets: &Assets, resolution: u32) -> Self {
        // Load the environment compute shader
        let compute = assets
            .load::<ComputeModule>("engine/shaders/scene/environment/environment.comp")
            .unwrap();

        // Create the bind layout for the compute shader
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("matrices");
        compiler.use_constant(0, resolution);
        compiler.use_storage_texture::<LayeredTexture2D<RGBA<f32>>>("enviro", StorageAccess::WriteOnly);
        compiler.use_push_constant_layout(graphics::PushConstantLayout::single(size_of::<f32>() * 4, ModuleVisibility::Compute).unwrap());
        let environment_shader = ComputeShader::new(compute, &compiler).unwrap();

        // Load the diffuse IBL convolution shader
        let compute = assets
            .load::<ComputeModule>("engine/shaders/scene/environment/diffuse.comp")
            .unwrap();

        // Create the bind layout for the compute shader
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_uniform_buffer::<vek::Vec4<vek::Vec4<f32>>>("matrices");
        compiler.use_constant(0, resolution / 16);
        compiler.use_storage_texture::<LayeredTexture2D<RGBA<f32>>>("diffuse", StorageAccess::WriteOnly);
        compiler.use_sampled_texture::<EnvironmentMap>("enviro", false);
        compiler.use_sampler::<RGBA<f32>>("enviro_sampler", false);
        let ibl_diffuse_convolution_shader = ComputeShader::new(compute, &compiler).unwrap();

        // Convert the eqilateral texture to a cubemap texture
        let projection =
            vek::Mat4::perspective_fov_rh_no(90.0f32.to_radians(), 1.0, 1.0, 0.02, 20.0);
        use vek::Mat4;
        use vek::Vec3;

        // View matrices for the 6 different faces
        let views: [Mat4<f32>; 6] = [
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_x(), -Vec3::unit_y()), // Left
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_x(), -Vec3::unit_y()), // Right
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_y(), Vec3::unit_z()),  // Top
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_y(), -Vec3::unit_z()), // Bottom
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_z(), -Vec3::unit_y()), // Back
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_z(), -Vec3::unit_y()), // Front
        ];

        // Multiply both matrices
        let matrices = views.map(|x| (projection * x).cols);

        // Create the uniform buffer that contains the matrices
        let matrices = UniformBuffer::<vek::Vec4<vek::Vec4<f32>>>::from_slice(
            graphics,
            &matrices,
            BufferMode::Dynamic,
            BufferUsage::empty()
        ).unwrap();

        Self {
            resolution, 
            environment_map: create_cubemap(graphics, resolution),

            matrices,

            environment_shader,
            diffuse_ibl_map: create_cubemap(graphics, resolution / 16),
            ibl_diffuse_convolution_shader,
        }
    }
}
