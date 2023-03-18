use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputePass, ComputeShader, Graphics,
    Normalized, SamplerSettings, Texture, Texture3D, TextureMipMaps,
    TextureMode, TextureUsage, R, RGBA,
};

// This mesh generator will take in the voxel data given from the voxel texture
// and will use a compute shader that will utilize the surface nets algorithm
// to generate an appropriate mesh for a chunk
pub struct MeshGenerator {
    pub shader: ComputeShader,
    pub positions: Texture3D<RGBA<Normalized<u8>>>,
}

impl MeshGenerator {
    // Create a new mesh generator to be used with the terrain system
    pub(crate) fn new(graphics: &Graphics, assets: &Assets) -> Self {
        // Load the mesh compute shader
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/mesh.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let compiler = Compiler::new(assets, graphics);

        // Compile the compute shader
        let compute = ComputeShader::new(module, compiler).unwrap();

        // Create a 3D texture that will contain the local positions of the SurfaceNets vertices
        let positions =
            Texture3D::<RGBA<Normalized<u8>>>::from_texels(
                graphics,
                None,
                vek::Extent3::broadcast(32),
                TextureMode::Dynamic,
                TextureUsage::STORAGE,
                SamplerSettings::default(),
                TextureMipMaps::Disabled,
            )
            .unwrap();

        todo!()
    }
}

// This will be responsible for calling a compute shader that will create the voxel data
// and store it within a texture
pub struct VoxelGenerator {
    pub shader: ComputeShader,
    pub densities: Texture3D<R<f32>>,
}

impl VoxelGenerator {
    // Create a new voxel generator to be used with the terrain system
    pub(crate) fn new(graphics: &Graphics, assets: &Assets) -> Self {
        // Load the voxel compute shader
        let module = assets
            .load::<ComputeModule>(
                "engine/shaders/terrain/voxel.comp",
            )
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_storage_texture::<Texture3D<R<f32>>>(
            "densities",
            false,
            true,
        );

        // Compile the compute shader
        let shader = ComputeShader::new(module, compiler).unwrap();

        // Create a 3D texture that will contain the local positions of the SurfaceNets vertices
        let densities = Texture3D::<R<f32>>::from_texels(
            graphics,
            None,
            vek::Extent3::broadcast(64),
            TextureMode::Dynamic,
            TextureUsage::STORAGE,
            SamplerSettings::default(),
            TextureMipMaps::Disabled,
        )
        .unwrap();

        Self { shader, densities }
    }
}
