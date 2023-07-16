use assets::Assets;

use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule, ComputeShader, GpuPod, Graphics,
    ModuleVisibility, StorageAccess, Texel, Texture3D, TextureUsage, Vertex, R, RG, XY, XYZW,
};

use crate::{create_texture3d, TempTriangles, TempVertices, TerrainRenderingMode, TerrainSettings};

// Mesh generator that will be solely used to generate the mesh from voxels
pub struct MeshGenerator {
    pub(crate) temp_vertices: TempVertices,
    pub(crate) temp_triangles: TempTriangles,
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) compute_quads: ComputeShader,
    pub(crate) cached_indices: Texture3D<R<u32>>,
}

impl MeshGenerator {
    // Create a new mesh generator that will generate our meshes
    pub(crate) fn new(assets: &Assets, graphics: &Graphics, settings: &TerrainSettings) -> Self {
        // Create some temporary vertices that we will write to first
        // Note: These should be able to handle a complex mesh in the worst case scenario
        let temp_vertices = Buffer::<<XYZW<f32> as Vertex>::Storage>::zeroed(
            graphics,
            (settings.mesher.size as usize).pow(3),
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::READ | BufferUsage::WRITE,
        )
        .unwrap();

        // Create some temporary triangles that we will write to first
        // Note: These should be able to handle a complex mesh in the worst case scenario
        let temp_triangles = Buffer::<[u32; 3]>::zeroed(
            graphics,
            (settings.mesher.size as usize - 1).pow(3) * 2,
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::READ | BufferUsage::WRITE,
        )
        .unwrap();

        // Load the compute module for vertex generation
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/vertices.comp")
            .unwrap();
        let mut compiler = Compiler::new(assets, graphics);

        compiler.use_push_constant_layout(
            graphics::PushConstantLayout::single(
                u32::size() + f32::size(),
                ModuleVisibility::Compute,
            )
            .unwrap(),
        );

        // Set the voxels texture that we will sample
        compiler.use_sampled_texture::<Texture3D<RG<f32>>>("voxels", false);
        compiler.use_sampler::<RG<f32>>("voxels_sampler", false);

        // Set the cached indices that we will use to reuse vertices
        compiler
            .use_storage_texture::<Texture3D<R<u32>>>("cached_indices", StorageAccess::WriteOnly);

        // Set storage buffers and counters
        compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
            "vertices",
            StorageAccess::WriteOnly,
        );
        compiler.use_storage_buffer::<[u32; 2]>("counters", StorageAccess::ReadWrite);

        // Set vertex generation parameters (constants)
        compiler.use_constant(0, settings.mesher.size);
        compiler.use_constant(
            1,
            matches!(settings.rendering.mode, TerrainRenderingMode::Blocky),
        );

        // Create the compute vertices shader
        let compute_vertices = ComputeShader::new(module, &compiler).unwrap();

        // Load the comput module for quads generation
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/quads.comp")
            .unwrap();
        let mut compiler = Compiler::new(assets, graphics);

        // Set the voxels texture that we will sample
        compiler.use_storage_texture::<Texture3D<RG<f32>>>("voxels", StorageAccess::ReadOnly);

        // Set the cached indices that we will use to reuse vertices
        compiler
            .use_storage_texture::<Texture3D<R<u32>>>("cached_indices", StorageAccess::ReadOnly);

        // Set counters and storage buffers
        compiler.use_storage_buffer::<[u32; 2]>("counters", StorageAccess::ReadWrite);
        compiler.use_storage_buffer::<u32>("triangles", StorageAccess::WriteOnly);

        // Set size constants
        compiler.use_constant(0, settings.mesher.size);

        // Create the compute quads shader
        let compute_quads = ComputeShader::new(module, &compiler).unwrap();

        Self {
            temp_vertices,
            temp_triangles,
            compute_vertices,
            compute_quads,
            cached_indices: create_texture3d(
                graphics,
                settings.mesher.size,
                TextureUsage::STORAGE | TextureUsage::WRITE,
                None,
            ),
        }
    }
}
