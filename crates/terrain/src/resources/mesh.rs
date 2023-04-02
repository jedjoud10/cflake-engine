use ahash::{AHashMap, AHashSet};
use assets::Assets;
use ecs::Entity;
use graphics::{
    Buffer, BufferMode, BufferUsage, Compiler, ComputeModule,
    ComputePass, ComputeShader, DrawIndexedIndirect,
    DrawIndexedIndirectBuffer, GpuPod, Graphics, ModuleVisibility,
    Normalized, PushConstantLayout, SamplerSettings, Texel, Texture,
    Texture3D, TextureMipMaps, TextureMode, TextureUsage,
    TriangleBuffer, Vertex, VertexBuffer, R, RGBA, XYZ, XYZW,
};
use rendering::{
    attributes, AttributeBuffer, IndirectMesh, MaterialId, Mesh,
    Pipelines,
};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial, TerrainSettings, create_texture3d, create_counters};

// Mesh generator that will be solely used to generate the mesh from voxels
pub struct MeshGenerator {
    pub(crate) temp_vertices: Buffer<<XYZW<f32> as Vertex>::Storage>,
    pub(crate) temp_triangles: Buffer<[u32; 3]>,
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) compute_quads: ComputeShader,
    pub(crate) cached_indices: Texture3D<R<u32>>,
    pub(crate) counters: Buffer<u32>,
}

impl MeshGenerator {
    // Create a new mesh generator that will generate our meshes
    pub(crate) fn new(assets: &Assets, graphics: &Graphics, settings: &TerrainSettings) -> Self {
        // Create some temporary vertices that we will write to first
        // Note: These should be able to handle a complex mesh in the worst case scenario
        let temp_vertices = Buffer::<<XYZW<f32> as Vertex>::Storage>::zeroed(
            graphics,
            (settings.size as usize).pow(3),
            BufferMode::Dynamic,
            BufferUsage::STORAGE,
        ).unwrap();

        // Create some temporary triangles that we will write to first
        // Note: These should be able to handle a complex mesh in the worst case scenario
        let temp_triangles = Buffer::<[u32; 3]>::zeroed(
            graphics,
            (settings.size as usize - 1).pow(3) * 2,
            BufferMode::Dynamic,
            BufferUsage::STORAGE,
        ).unwrap();

        // Load the compute module for vertex generation
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/vertices.comp")
            .unwrap();
        let mut compiler = Compiler::new(assets, graphics);

        // Set the densitites texture that we will sample
        compiler.use_storage_texture::<Texture3D<R<f32>>>(
            "densities",
            true,
            false,
        );

        // Set the cached indices that we will use to reuse vertices
        compiler.use_storage_texture::<Texture3D<R<u32>>>(
            "cached_indices",
            false,
            true,
        );

        // Set storage buffers and counters
        compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
            "vertices", false, true,
        );
        compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);

        // Set vertex generation parameters (constants)
        compiler.use_constant(0, settings.size);
        compiler.use_constant(1, settings.smoothing);

        // Create the compute vertices shader
        let compute_vertices = ComputeShader::new(module, compiler).unwrap();

        // Load the comput module for quads generation
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/quads.comp")
            .unwrap();
        let mut compiler = Compiler::new(assets, graphics);

        // Set the densitites texture that we will sample
        compiler.use_storage_texture::<Texture3D<R<f32>>>(
            "densities",
            true,
            false,
        );

        // Set the cached indices that we will use to reuse vertices
        compiler.use_storage_texture::<Texture3D<R<u32>>>(
            "cached_indices",
            true,
            false,
        );

        // Set counters and storage buffers
        compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);
        compiler.use_storage_buffer::<u32>("triangles", false, true);

        // Set size constants
        compiler.use_constant(0, settings.size);

        // Create the compute quads shader
        let compute_quads = ComputeShader::new(module, compiler).unwrap();

        Self {
            temp_vertices,
            temp_triangles,
            compute_vertices,
            compute_quads,
            cached_indices: create_texture3d(graphics, settings.size),
            counters: create_counters(graphics, 2),
        }
    }
}