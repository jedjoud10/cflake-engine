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
use rendering::{MaterialId, Mesh, Pipelines};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial};

// Type aliases for textures and buffers
pub(crate) type CachedIndices = Texture3D<R<u32>>;
pub(crate) type Densities = Texture3D<R<f32>>;
pub(crate) type Counters = Buffer<u32>;

// Le terrain generator
// TODO: EXPLAIN
pub struct Terrain {
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
    pub(crate) chunks: AHashSet<ChunkCoords>,
    pub(crate) entities: AHashMap<ChunkCoords, Entity>,
    pub(crate) size: u32,
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Keep a pool of all meshes and indirect buffers
    pub(crate) meshes: Vec<(Handle<Mesh>, bool)>,
    pub(crate) indirect_buffers:
        Vec<(Handle<DrawIndexedIndirectBuffer>, bool)>,
    pub(crate) chunk_render_distance: u32,

    // Location of the chunk viewer
    pub(crate) viewer: Option<(Entity, ChunkCoords)>,
}

impl Terrain {
    // Create a new mesh generator to be used with the terrain system
    pub fn new(
        graphics: &Graphics,
        assets: &Assets,
        size: u32,
        chunk_render_distance: u32,
        smoothing: bool,
        meshes: &mut Storage<Mesh>,
        indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        materials: &mut Storage<TerrainMaterial>,
        pipelines: &mut Pipelines,
    ) -> Self {
        // Load the required compute shaders (not faillible)
        let compute_voxels =
            load_compute_voxels_shaders(assets, graphics);
        let compute_vertices = load_compute_vertices_shader(
            assets, graphics, size, smoothing,
        );
        let compute_quads =
            load_compute_quads_shader(assets, graphics, size);

        // Create cached data used for generation
        let cached_indices = create_texture3d(graphics, size);
        let counters = create_counters(graphics);
        let densities = create_texture3d(graphics, size);

        // Pre-allocate the meshes and indirect buffers
        let meshes = preallocate_meshes(
            graphics,
            meshes,
            chunk_render_distance,
            size,
        );
        let indirect_buffers = preallocate_indirect_buffers(
            graphics,
            indirect_buffers,
            chunk_render_distance,
        );

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = (size) / 4;

        Self {
            compute_vertices,
            compute_quads,
            cached_indices,
            counters,
            dispatch,
            densities,
            compute_voxels,
            chunks: Default::default(),
            entities: Default::default(),
            size,
            material: materials.insert(TerrainMaterial {
                bumpiness: 0.1,
                roughness: 1.0,
                metallic: 0.0,
                ambient_occlusion: 0.0,
            }),
            id: pipelines.register(graphics, assets).unwrap(),
            viewer: None,
            meshes,
            indirect_buffers,
            chunk_render_distance,
        }
    }
}

// Create the indirect buffers for indirect surfaces before hand
fn preallocate_indirect_buffers(
    graphics: &Graphics,
    indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
    render_distance: u32,
) -> Vec<(Handle<DrawIndexedIndirectBuffer>, bool)> {
    let count = (render_distance * 2 + 1).pow(3);

    let mut vec = Vec::new();
    for _ in 0..count {
        let indirect = DrawIndexedIndirectBuffer::from_slice(
            graphics,
            &[DrawIndexedIndirect {
                vertex_count: 0,
                instance_count: 1,
                base_index: 0,
                vertex_offset: 0,
                base_instance: 0,
            }],
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE,
        )
        .unwrap();

        let handle = indirect_buffers.insert(indirect);
        vec.push((handle, true));
    }
    vec
}

// Create the meshes that we will use for terrain generation before hand
fn preallocate_meshes(
    graphics: &Graphics,
    meshes: &mut Storage<Mesh>,
    render_distance: u32,
    size: u32,
) -> Vec<(Handle<Mesh>, bool)> {
    let count = (render_distance * 2 + 1).pow(2) * 2;

    let mut vec = Vec::new();

    for _ in 0..count {
        // Calculate the maximum number of vertices that we can store
        let vertex_count = (size as usize).pow(3);
        let triangle_count = (size as usize - 1).pow(3) * 3;

        // Create the vertex buffer (make sure size can contain ALL possible vertices)
        let vertices = VertexBuffer::<XYZW<f32>>::zeroed(
            graphics,
            vertex_count,
            BufferMode::Dynamic,
            BufferUsage::STORAGE,
        )
        .unwrap();

        // Create the normal buffer (make sure size can contain ALL possible normals)
        let normals = VertexBuffer::<XYZW<Normalized<i8>>>::zeroed(
            graphics,
            vertex_count,
            BufferMode::Dynamic,
            BufferUsage::STORAGE,
        )
        .unwrap();

        // Create the triangle buffer (make sure size can contain ALL possible triangles)
        let triangles = TriangleBuffer::<u32>::zeroed(
            graphics,
            triangle_count,
            BufferMode::Dynamic,
            BufferUsage::STORAGE,
        )
        .unwrap();

        // Create a mesh that uses the buffers
        let mesh = Mesh::from_buffers(
            Some(vertices),
            Some(normals),
            None,
            None,
            triangles,
        )
        .unwrap();
        vec.push((meshes.insert(mesh), true));
    }
    vec
}

// Create counters that will help us generate the vertices
fn create_counters(graphics: &Graphics) -> Buffer<u32> {
    // Create the atomic counter buffer
    let counters = Buffer::from_slice(
        graphics,
        &[0, 0],
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE,
    )
    .unwrap();
    counters
}

// Create a 3D storage texture with null contents with the specified size
fn create_texture3d<T: Texel>(
    graphics: &Graphics,
    size: u32,
) -> Texture3D<T> {
    Texture3D::<T>::from_texels(
        graphics,
        None,
        vek::Extent3::broadcast(size),
        TextureMode::Dynamic,
        TextureUsage::STORAGE,
        SamplerSettings::default(),
        TextureMipMaps::Disabled,
    )
    .unwrap()
}

// Load the compute shader that will generate quads
fn load_compute_quads_shader(
    assets: &Assets,
    graphics: &Graphics,
    size: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/quads.comp")
        .unwrap();
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_texture::<Densities>(
        "densities",
        true,
        false,
    );
    compiler.use_storage_texture::<CachedIndices>(
        "cached_indices",
        true,
        false,
    );
    compiler.use_storage_buffer::<u32>("triangles", false, true);
    compiler.use_storage_buffer::<DrawIndexedIndirect>(
        "indirect", true, true,
    );
    compiler
        .use_snippet("size", format!("const uint size = {size};"));
    let compute_quads = ComputeShader::new(module, compiler).unwrap();
    compute_quads
}

// Load the compute shader that will generate vertex positions
fn load_compute_vertices_shader(
    assets: &Assets,
    graphics: &Graphics,
    size: u32,
    smoothing: bool,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/vertices.comp")
        .unwrap();
    let mut compiler = Compiler::new(assets, graphics);

    // Set the required shader resources
    compiler.use_storage_texture::<Densities>(
        "densities",
        true,
        false,
    );
    compiler.use_storage_texture::<CachedIndices>(
        "cached_indices",
        true,
        true,
    );
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>(
        "vertices", false, true,
    );
    compiler.use_storage_buffer::<<XYZW<Normalized<i8>> as Vertex>::Storage>("normals", false, true);
    compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);
    compiler
        .use_snippet("size", format!("const uint size = {size};"));
    compiler.use_snippet(
        "smoothing",
        format!("const bool smoothing = {};", smoothing),
    );
    let compute_vertices =
        ComputeShader::new(module, compiler).unwrap();
    compute_vertices
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
    compiler.use_storage_texture::<Texture3D<R<f32>>>(
        "densities",
        false,
        true,
    );

    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <vek::Vec4<vek::Vec4<f32>> as GpuPod>::size() * 2,
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );

    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}
