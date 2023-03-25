use ahash::AHashSet;
use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputePass, ComputeShader, Graphics,
    Normalized, SamplerSettings, Texture, Texture3D, TextureMipMaps,
    TextureMode, TextureUsage, R, RGBA, Buffer, TriangleBuffer, VertexBuffer, XYZ, XYZW, Vertex, BufferMode, BufferUsage, DrawIndexedIndirectBuffer, DrawIndexedIndirect, PushConstantLayout, ModuleVisibility, Texel,
};
use rendering::Mesh;
use utils::{Handle, Storage};

use crate::ChunkCoords;

// Type aliases for textures and buffers
pub(crate) type CachedIndices = Texture3D<R<u32>>;
pub(crate) type Densities = Texture3D<R<f32>>;
pub(crate) type Counters = Buffer<u32>;

// Le terrain generator
// TODO: EXPLAIN
pub struct ProceduralTerrain {
    // This will be responsible for filling up the "densities" texture with proper density data
    pub(crate) compute_quads: ComputeShader,
    pub(crate) densities: Densities,

    // These mesh shaders will take in the voxel data given from the voxel texture
    // and will use a compute shader that will utilize the surface nets algorithm
    // to generate an appropriate mesh for a chunk
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) cached_indices: CachedIndices,
    pub(crate) counters: Counters,
    
    // All compute shaders use the same local dispatch work group size
    pub(crate) dispatch: u32,

    // Terrain generator will also be responsible for chunks
    pub(crate) chunks: AHashSet<ChunkCoords>
}

impl ProceduralTerrain {
    // Create a new mesh generator to be used with the terrain system
    pub fn new(
        graphics: &Graphics,
        assets: &Assets,
        size: u32
    ) -> Self {
        // Load the required compute shaders (not faillible)
        let compute_voxels = load_compute_voxels_shaders(assets, graphics);
        let compute_vertices = load_compute_vertices_shader(assets, graphics);
        let compute_quads = load_compute_quads_shader(assets, graphics);

        // Create cached data used for generation
        let cached_indices = create_texture3d(graphics, size);
        let counters = create_counters(graphics);
        let densities = create_texture3d(graphics, size);

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = size / 4;

        Self {
            compute_vertices,
            compute_quads,
            cached_indices,
            counters,
            dispatch,
            densities,
            compute_voxels,
            chunks: Default::default(),
        }
    }
}

// Create counters that will help us generate the vertices
fn create_counters(graphics: &Graphics) -> Buffer<u32> {
    // Create the atomic counter buffer
    let counters = Buffer::from_slice(
        graphics,
        &[0, 0],
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE
    ).unwrap();
    counters
}

// Create a 3D storage texture with null contents with the specified size 
fn create_texture3d<T: Texel>(graphics: &Graphics, size: u32) -> Texture3D<T> {
    Texture3D::<T>::from_texels(
        graphics,
        None,
        vek::Extent3::broadcast(size),
        TextureMode::Dynamic,
        TextureUsage::STORAGE,
        SamplerSettings::default(),
        TextureMipMaps::Disabled,
    ).unwrap()
}

// Load the compute shader that will generate quads
fn load_compute_quads_shader(assets: &Assets, graphics: &Graphics) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/quads.comp")
        .unwrap();
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_texture::<Densities>("densities", true, false);
    compiler.use_storage_texture::<CachedIndices>("cached_indices", true, false);
    compiler.use_storage_buffer::<u32>("triangles", false, true);
    compiler.use_storage_buffer::<DrawIndexedIndirect>("indirect", true, true);
    let compute_quads = ComputeShader::new(module, compiler).unwrap();
    compute_quads
}

// Load the compute shader that will generate vertex positions
fn load_compute_vertices_shader(assets: &Assets, graphics: &Graphics) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/vertices.comp")
        .unwrap();
    let mut compiler = Compiler::new(assets, graphics);
        
    // Set the required shader resources
    compiler.use_storage_texture::<Densities>("densities", true, false);
    compiler.use_storage_texture::<CachedIndices>("cached_indices", true, true);
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>("vertices", false, true);
    compiler.use_storage_buffer::<<XYZW<Normalized<i8>> as Vertex>::Storage>("normals", false, true);
    compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);
    let compute_vertices = ComputeShader::new(module, compiler).unwrap();
    compute_vertices
}

// Load the voxel compute shader
fn load_compute_voxels_shaders(assets: &Assets, graphics: &Graphics) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>(
            "engine/shaders/terrain/voxels.comp",
        )
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_texture::<Texture3D<R<f32>>>(
        "densities",
        false,
        true,
    );

    compiler.use_push_constant_layout(PushConstantLayout::single(4, ModuleVisibility::Compute).unwrap());

    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}