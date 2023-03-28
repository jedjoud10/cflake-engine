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
use rendering::{MaterialId, Mesh, Pipelines, IndirectMesh, AttributeBuffer, attributes};
use utils::{Handle, Storage};

use crate::{ChunkCoords, TerrainMaterial};

// Type aliases for textures and buffers
pub(crate) type CachedIndices = Texture3D<R<u32>>;
pub(crate) type Densities = Texture3D<R<f32>>;
pub(crate) type Counters = Buffer<[u32; 2]>;

// Terrain generator settings that the user will need to add to configure the terrain gen
pub struct TerrainSettings {
    pub size: u32,
    pub chunk_render_distance: u32,
    pub smoothing: bool,
}

// Plan:
// allocate storages buffers of 128mb each
// bind all buffers to the same compute pass
// bind temp vert+triangle buffer that will store worst case scenario vertices and tris
// run voxel compute shader
// run mesh compute shader
// run "guesser" compute that will try to find free memory location based on last draw indexed indirect vertex count and shiz
// run copy compute shader that will copy temp memory to the free one
// gg ez but no dynamic chunks cause hard for me

// Le terrain generator
// TODO: EXPLAIN
// TODO: Split this into smaller structs
pub struct Terrain {
    // This will be responsible for filling up the "densities" texture with proper density data
    pub(crate) compute_quads: ComputeShader,
    pub(crate) densities: Densities,

    // Container for all the memory chunks that we pre-allocate
    pub(crate) shared_vertex_buffer: Handle<AttributeBuffer<attributes::Position>>,
    pub(crate) shared_triangle_buffer: Handle<TriangleBuffer<u32>>,

    // Temporary buffers where we will write the mesh data
    pub(crate) temp_vertices: VertexBuffer<XYZW<f32>>,
    pub(crate) temp_triangles: TriangleBuffer<u32>,

    // These mesh shaders will take in the voxel data given from the voxel texture
    // and will use a compute shader that will utilize the surface nets algorithm
    // to generate an appropriate mesh for a chunk
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) cached_indices: CachedIndices,
    pub(crate) old_counters: Counters,
    pub(crate) new_counters: Counters,
    pub(crate) current_counters: Counters,

    // Guesser compute shader that will look into the memory chunks and 
    // try to find a free chunk of memory that we can use
    /*
    pub(crate) compute_try_find: ComputeShader,
    */

    // Copy shader that will copy the temporary vertices and triangles to the
    // output vertices and shaders within each block 
    pub(crate) compute_copy: ComputeShader,

    // All compute shaders use the same local dispatch work group size
    pub(crate) dispatch: u32,

    // Terrain generator will also be responsible for chunks
    pub(crate) chunks: AHashSet<ChunkCoords>,
    pub(crate) entities: AHashMap<ChunkCoords, Entity>,
    pub(crate) size: u32,
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Keep a pool of all meshes and indirect buffers
    pub(crate) indirect_meshes: Vec<(Handle<IndirectMesh>, bool)>,
    pub(crate) chunk_render_distance: u32,

    // Location of the chunk viewer
    pub(crate) viewer: Option<(Entity, ChunkCoords)>,
}

impl Terrain {
    // Create a new mesh generator to be used with the terrain system
    pub(crate) fn new(
        graphics: &Graphics,
        assets: &Assets,
        settings: TerrainSettings,
        indirect_meshes: &mut Storage<IndirectMesh>,
        indirect_buffers: &mut Storage<DrawIndexedIndirectBuffer>,
        vertices: &mut Storage<AttributeBuffer<attributes::Position>>,
        triangles: &mut Storage<TriangleBuffer<u32>>,
        materials: &mut Storage<TerrainMaterial>,
        pipelines: &mut Pipelines,
    ) -> Self {
        let TerrainSettings {
            size,
            chunk_render_distance,
            smoothing,
        } = settings;

        let shared_vertex_buffer_size = graphics.device().limits().max_storage_buffer_binding_size / 4 / 4;
        let shared_triangle_buffer_size = graphics.device().limits().max_storage_buffer_binding_size / 4 / 3;

        // Load the required generationcompute shaders
        let compute_voxels =
            load_compute_voxels_shaders(assets, graphics);
        let compute_vertices = load_compute_vertices_shader(
            assets, graphics, size, smoothing,
        );
        let compute_quads =
            load_compute_quads_shader(assets, graphics, size);
        let compute_copy = 
            load_compute_copy_shader(assets, graphics,  shared_triangle_buffer_size, shared_vertex_buffer_size, size);

        // Create cached data used for generation
        let cached_indices = create_texture3d(graphics, size);
        let old_counters = create_counters(graphics);
        let new_counters = create_counters(graphics);
        let current_counters = create_counters(graphics);
        let densities = create_texture3d(graphics, size);
        let temp_vertices = create_temp_vertices(graphics, size);
        let temp_triangles = create_temp_triangles(graphics, size);

        // Create the memory chunk vertices
        let shared_vertex_buffer = create_shared_vertex_buffer(
            graphics,
            vertices,
            shared_vertex_buffer_size
        );

        // Create the memory chunk triangles
        let shared_triangle_buffer = create_shared_triangle_buffer(
            graphics,
            triangles,
            shared_triangle_buffer_size
        );

        // Pre-allocate the meshes and indirect buffers
        let indirect_meshes = preallocate_meshes(
            graphics,
            shared_vertex_buffer.clone(),
            shared_triangle_buffer.clone(),
            indirect_meshes,
            indirect_buffers,
            chunk_render_distance,
            size,
        );

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = (size) / 4;

        Self {
            compute_vertices,
            compute_quads,
            cached_indices,
            old_counters,
            new_counters,
            current_counters,
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
            indirect_meshes,
            chunk_render_distance,
            
            shared_vertex_buffer,
            shared_triangle_buffer,

            temp_vertices,
            temp_triangles,
            compute_copy,
        }
    }
}


// Creates one big triangle buffer that will contain all of our data
fn create_shared_triangle_buffer(
    graphics: &Graphics,
    triangles: &mut Storage<TriangleBuffer<u32>>,
    triangle_buffer_size: u32,
) -> Handle<TriangleBuffer<u32>> {
    triangles.insert(TriangleBuffer::zeroed(
        graphics,
        triangle_buffer_size as usize,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE
    ).unwrap())
}

// Creates one big vertex buffer that will contain all of our data
fn create_shared_vertex_buffer(
    graphics: &Graphics,
    vertices: &mut Storage<AttributeBuffer<attributes::Position>>,
    vertex_buffer_size: u32,
) -> Handle<AttributeBuffer<attributes::Position>> {
    vertices.insert(AttributeBuffer::<attributes::Position>::zeroed(
        graphics,
        vertex_buffer_size as usize,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE
    ).unwrap())
}

// Create some temporary triangles that we will write to first
// Note: These should be able to handle a complex mesh in the worst case scenario
fn create_temp_triangles(graphics: &Graphics, size: u32) -> TriangleBuffer<u32> {
    TriangleBuffer::zeroed(
        graphics,
        (size as usize - 1).pow(3) * 2,
        BufferMode::Dynamic,
        BufferUsage::STORAGE
    ).unwrap()
}

// Create some temporary vertices that we will write to first
// Note: These should be able to handle a complex mesh in the worst case scenario
fn create_temp_vertices(graphics: &Graphics, size: u32) -> VertexBuffer<XYZW<f32>> {
    AttributeBuffer::<attributes::Position>::zeroed(
        graphics,
        (size as usize).pow(3),
        BufferMode::Dynamic,
        BufferUsage::STORAGE
    ).unwrap()
}

// Create the meshes that we will use for terrain generation before hand
fn preallocate_meshes(
    graphics: &Graphics,
    shared_vertex_buffer: Handle<AttributeBuffer<attributes::Position>>,
    shared_triangle_buffer: Handle<TriangleBuffer<u32>>,
    meshes: &mut Storage<IndirectMesh>,
    buffers: &mut Storage<DrawIndexedIndirectBuffer>,
    render_distance: u32,
    chunk_size: u32,
) -> Vec<(Handle<IndirectMesh>, bool)> {
    let count = (render_distance * 2 + 1).pow(3);

    (0..count).into_iter().map(|_| {
        let indirect = buffers.insert(DrawIndexedIndirectBuffer::from_slice(
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

        let mut mesh = IndirectMesh::from_handles(
            Some(shared_vertex_buffer.clone()),
            None,
            None,
            None,
            shared_triangle_buffer.clone(),
            indirect
        );

        mesh.set_aabb(Some(math::Aabb {
            min: vek::Vec3::zero(),
            max: vek::Vec3::one() * chunk_size as f32,
        }));

        let handle = meshes.insert(mesh);

        (handle, true)
    }).collect()
}

// Create counters that will help us generate the vertices
fn create_counters(graphics: &Graphics) -> Buffer<[u32; 2]> {
    // Create the atomic counter buffer
    let counters = Buffer::from_slice(
        graphics,
        &[[0, 0]],
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST | BufferUsage::WRITE,
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
    compiler.use_storage_buffer::<[u32; 2]>("counters", true, true);
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

// Load the compute shader that will find a free memory range
fn load_compute_try_find_shader(
    assets: &Assets,
    graphics: &Graphics,
    size: u32,
    memory_chunks_count: u32,
) -> ComputeShader {
    todo!()
}

// Load the compute shader that will copy the temp data to perm allocation space
fn load_compute_copy_shader(
    assets: &Assets,
    graphics: &Graphics,
    output_triangles_count: u32,
    output_vertices_count: u32,
    size: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/copy.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_buffer::<[u32; 2]>("current_counters", true, true);
    compiler.use_storage_buffer::<[u32; 2]>("old_counters", true, true);
    compiler.use_storage_buffer::<[u32; 2]>("new_counters", true, true);
    compiler
        .use_snippet("size", format!("const uint size = {size};"));
    compiler
        .use_snippet("output_triangles_count", format!("const uint output_triangles_count = {output_triangles_count};"));
    compiler
        .use_snippet("output_vertices_count", format!("const uint output_vertices_count = {output_vertices_count};"));
    compiler.use_storage_buffer::<DrawIndexedIndirect>(
        "indirect", false, true,
    );
    
    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}