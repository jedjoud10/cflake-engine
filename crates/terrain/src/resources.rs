use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputePass, ComputeShader, Graphics,
    Normalized, SamplerSettings, Texture, Texture3D, TextureMipMaps,
    TextureMode, TextureUsage, R, RGBA, Buffer, TriangleBuffer, VertexBuffer, XYZ, XYZW, Vertex, BufferMode, BufferUsage,
};

// Type aliases for textures and buffers
pub(crate) type CachedIndices = Texture3D<R<u32>>;
pub(crate) type Vertices = VertexBuffer<XYZW<u8>>;
pub(crate) type Triangles = TriangleBuffer<u32>;
pub(crate) type Densities = Texture3D<R<f32>>;
pub(crate) type Counters = Buffer<u32>;


// This mesh generator will take in the voxel data given from the voxel texture
// and will use a compute shader that will utilize the surface nets algorithm
// to generate an appropriate mesh for a chunk
pub struct MeshGenerator {
    pub(crate) shader: ComputeShader,
    pub(crate) cached_indices: CachedIndices,
    pub(crate) vertices: Vertices,
    pub(crate) triangles: Triangles,
    pub(crate) counters: Counters,
    pub(crate) dispatch: u32,
}

impl MeshGenerator {
    // Create a new mesh generator to be used with the terrain system
    pub(crate) fn new(graphics: &Graphics, assets: &Assets, size: u32) -> Self {
        // Load the mesh compute shader
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/mesh.comp")
            .unwrap();

        // Create a simple compute shader compiler
        let mut compiler = Compiler::new(assets, graphics);
        
        // Set the required shader resources
        compiler.use_storage_texture::<Densities>("densities", true, false);
        compiler.use_storage_texture::<CachedIndices>("cached_indices", true, true);
        compiler.use_storage_buffer::<<XYZW<u8> as Vertex>::Storage>("vertices", false, true);
        compiler.use_storage_buffer::<[u32; 3]>("triangles", false, true);
        compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);

        // Compile the compute shader
        let compute = ComputeShader::new(module, compiler).unwrap();

        // Create cached indices atomic texture
        let cached_indices = CachedIndices::from_texels(
            graphics,
            None,
            vek::Extent3::broadcast(size),
            TextureMode::Dynamic,
            TextureUsage::STORAGE,
            SamplerSettings::default(),
            TextureMipMaps::Disabled,
        ).unwrap();

        // Create the vertex buffer (make sure size can contain ALL possible vertices)
        let vertex_capacity = (size as usize).pow(3);
        let vertices = Vertices::with_capacity(
            graphics, 
            vertex_capacity,
            BufferMode::Parital,
            BufferUsage::STORAGE
        ).unwrap();
        
        // Create the triangle buffer (make sure size can contain ALL possible triangles)
        let triangle_capacity = (size as usize - 1).pow(3) * 4;
        let triangles = Triangles::with_capacity(
            graphics, 
            triangle_capacity,
            BufferMode::Parital,
            BufferUsage::STORAGE
        ).unwrap();

        // Create the atomic counter buffer
        let counters = Buffer::from_slice(
            graphics,
            &[0, 0],
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap();

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = size / 4;

        Self {
            shader: compute,
            cached_indices,
            vertices,
            triangles,
            counters,
            dispatch,
        }
    }
}

// This will be responsible for calling a compute shader that will create the voxel data
// and store it within a texture
pub struct VoxelGenerator {
    pub(crate) shader: ComputeShader,
    pub(crate) densities: Densities,
    pub(crate) dispatch: u32,
}

impl VoxelGenerator {
    // Create a new voxel generator to be used with the terrain system
    pub(crate) fn new(graphics: &Graphics, assets: &Assets, size: u32) -> Self {
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
            vek::Extent3::broadcast(size),
            TextureMode::Dynamic,
            TextureUsage::STORAGE,
            SamplerSettings::default(),
            TextureMipMaps::Disabled,
        )
        .unwrap();

        // Calculate the dispatch size for voxel generation by assuming local size is 4
        let dispatch = size / 4;

        Self {
            shader,
            densities,
            dispatch,
        }
    }
}
