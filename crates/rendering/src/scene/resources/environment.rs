use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputeShader, CubeMap, Graphics, ImageTexel,
    LayeredTexture2D, SamplerSettings,
    StorageAccess, Texel, Texture, TextureMipMaps, TextureMode, TextureUsage, RGBA,
};

pub type EnvironmentMap = CubeMap<RGBA<f32>>;
pub type TempEnvironmentMap = LayeredTexture2D<RGBA<f32>>;

// Create a cubemap with a specific resolution
fn create_cubemap<T: Texel + ImageTexel>(
    graphics: &Graphics,
    value: T::Storage,
    resolution: usize,
) -> CubeMap<T> {
    CubeMap::<T>::from_texels(
        graphics,
        Some(&vec![value; resolution * resolution * 6]),
        vek::Extent2::broadcast(resolution as u32),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::TARGET | TextureUsage::COPY_DST,
        Some(SamplerSettings::default()),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Create a "fake" cubemap (just a layered texture 2D) with a specific resolution
fn create_temp_cubemap<T: Texel + ImageTexel>(
    graphics: &Graphics,
    value: T::Storage,
    resolution: usize,
) -> LayeredTexture2D<T> {
    LayeredTexture2D::<T>::from_texels(
        graphics,
        Some(&vec![value; resolution * resolution * 6]),
        (vek::Extent2::broadcast(resolution as u32), 6),
        TextureMode::Dynamic,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
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

    pub(crate) temp: TempEnvironmentMap,

    // Compute shader that will create the envinronment map
    //pub(crate) shader: ComputeShader,

    // Projection and view matrices
    views: [vek::Mat4<f32>; 6],
    projection: vek::Mat4<f32>,
}

impl Environment {
    // Create a new scene environment render passes and cubemaps
    pub(crate) fn new(graphics: &Graphics, assets: &Assets) -> Self {
        /*
        // Load the environment compute shader
        let compute = assets
            .load::<ComputeModule>("engine/shaders/scene/environment/environment.comp")
            .unwrap();

        // Create the bind layout for the compute shader
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_storage_texture::<TempEnvironmentMap>("enviro", StorageAccess::WriteOnly);
        let shader = ComputeShader::new(compute, &compiler).unwrap();
        */

        // Convert the eqilateral texture to a cubemap texture
        let projection =
            vek::Mat4::perspective_fov_rh_no(90.0f32.to_radians(), 1.0, 1.0, 0.02, 20.0);
        use vek::Mat4;
        use vek::Vec3;

        // View matrices for the 6 different faces
        let views: [Mat4<f32>; 6] = [
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_x(), -Vec3::unit_y()), // Right
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_x(), -Vec3::unit_y()), // Left
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_y(), Vec3::unit_z()),  // Top
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_y(), -Vec3::unit_z()), // Bottom
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_z(), -Vec3::unit_y()), // Back
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_z(), -Vec3::unit_y()), // Front
        ];

        Self {
            environment_map: [
                create_cubemap(graphics, vek::Vec4::zero(), 128),
                create_cubemap(graphics, vek::Vec4::zero(), 128),
            ],

            temp: create_temp_cubemap(graphics, vek::Vec4::zero(), 128),

            //shader,
            views,
            projection,
        }
    }
}
