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

use crate::{ChunkCoords, TerrainMaterial};

// Voxel generator that will be solely used for generating voxels 
pub struct VoxelGenerator {
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) densities: Texture3D<R<f32>>,
}




// Load the voxel compute shader
fn load_compute_voxels_shaders(
    assets: &Assets,
    graphics: &Graphics,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/voxels.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);

    // Use the 3D densities texture that we will write to
    compiler.use_storage_texture::<Texture3D<R<f32>>>(
        "densities",
        false,
        true,
    );

    // TODO: Create 3D color texture as well

    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <vek::Vec4<f32> as GpuPod>::size() + u32::size() * 2,
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );

    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}
