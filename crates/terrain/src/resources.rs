use assets::Assets;
use graphics::{
    Compiler, ComputeModule, ComputePass, ComputeShader, Graphics,
    Normalized, SamplerSettings, Texture, Texture3D, TextureMipMaps,
    TextureMode, TextureUsage, R, RGBA, Buffer, TriangleBuffer, VertexBuffer, XYZ, XYZW, Vertex, BufferMode, BufferUsage, DrawIndexedIndirectBuffer, DrawIndexedIndirect, PushConstantLayout, ModuleVisibility,
};
use rendering::Mesh;
use utils::{Handle, Storage};

// Type aliases for textures and buffers
pub(crate) type CachedIndices = Texture3D<R<u32>>;
pub(crate) type Densities = Texture3D<R<f32>>;
pub(crate) type Counters = Buffer<u32>;


// This mesh generator will take in the voxel data given from the voxel texture
// and will use a compute shader that will utilize the surface nets algorithm
// to generate an appropriate mesh for a chunk
pub struct MeshGenerator {
    pub(crate) compute_quads: ComputeShader,
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) cached_indices: CachedIndices,
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) counters: Counters,
    pub(crate) dispatch: u32,
    pub(crate) indirect: Handle<DrawIndexedIndirectBuffer>,
}

impl MeshGenerator {
    // Create a new mesh generator to be used with the terrain system
    pub(crate) fn new(
        graphics: &Graphics,
        assets: &Assets,
        indirect: &mut Storage<DrawIndexedIndirectBuffer>,
        mesh: &mut Storage<Mesh>,
        size: u32
    ) -> Self {
        // Load the compute shader that will generate vertex positions
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

        // Load the compute shader that will generate quads
        let module = assets
            .load::<ComputeModule>("engine/shaders/terrain/quads.comp")
            .unwrap();
        let mut compiler = Compiler::new(assets, graphics);
        compiler.use_storage_texture::<Densities>("densities", true, false);
        compiler.use_storage_texture::<CachedIndices>("cached_indices", true, false);
        compiler.use_storage_buffer::<u32>("triangles", false, true);
        compiler.use_storage_buffer::<DrawIndexedIndirect>("indirect", true, true);
        let compute_quads = ComputeShader::new(module, compiler).unwrap();

        // Create cached indices atomic texture
        let cached_indices = CachedIndices::from_texels(
            graphics,
            None,
            vek::Extent3::broadcast(size),
            TextureMode::Dynamic,
            TextureUsage::STORAGE | TextureUsage::WRITE,
            SamplerSettings::default(),
            TextureMipMaps::Disabled,
        ).unwrap();

        // Create the vertex buffer (make sure size can contain ALL possible vertices)
        let vertex_capacity = (size as usize).pow(3);
        let vertices = VertexBuffer::<XYZW<f32>>::zeroed(
            graphics, 
            vertex_capacity,
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap();

        // Create the normal buffer (make sure size can contain ALL possible normals)
        let normals = VertexBuffer::<XYZW<Normalized<i8>>>::zeroed(
            graphics, 
            vertex_capacity,
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap();
        
        // Create the triangle buffer (make sure size can contain ALL possible triangles)
        let triangle_capacity = (size as usize - 1).pow(3) * 4 * 6;
        let triangles = TriangleBuffer::<u32>::zeroed(
            graphics, 
            triangle_capacity,
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap();

        // Create a mesh that uses the buffers
        let mesh = mesh.insert(Mesh::from_buffers(
            Some(vertices),
            Some(normals),
            None,
            None,
            triangles
        ).unwrap());

        // Create the atomic counter buffer
        let counters = Buffer::from_slice(
            graphics,
            &[0, 0],
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE
        ).unwrap();

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = size / 4;

        // Create an indexed indirect draw buffer and add into the storage
        let indirect = indirect.insert(DrawIndexedIndirectBuffer::from_slice(
            graphics,
            &[DrawIndexedIndirect {
                vertex_count: 0,
                instance_count: 1,
                base_index: 0,
                vertex_offset: 0,
                base_instance: 0,
            }],
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE
        ).unwrap());
        Self {
            compute_vertices,
            compute_quads,
            cached_indices,
            counters,
            dispatch,
            indirect,
            mesh,
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

        compiler.use_push_constant_layout(PushConstantLayout::single(4, ModuleVisibility::Compute).unwrap());

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
