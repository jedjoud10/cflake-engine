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

// Terrain generator settings that the user will need to add to configure the terrain gen
pub struct TerrainSettings {
    pub size: u32,
    pub chunk_render_distance: u32,
    pub allocations: usize,
    pub smoothing: bool,
}

// Plan:
// allocate storages buffers of 128mb each
// bind all buffers to the same compute pass
// bind temp vert+triangle buffer that will store worst case scenario vertices and tris
// run voxel compute shader
// run mesh compute shader
// run "finder" compute that will try to find free memory location based on ranges and current counter values (to get size limits required)
// run copy compute shader that will copy temp memory to the free one
// gg ez

// Le terrain generator
// TODO: EXPLAIN
// TODO: Split this into smaller structs
pub struct Terrain {
    // This will be responsible for filling up the "densities" texture with proper density data
    pub(crate) compute_quads: ComputeShader,
    pub(crate) densities: Densities,

    // Container for all the memory chunks that we pre-allocate
    pub(crate) shared_vertex_buffers: Vec<Handle<AttributeBuffer<attributes::Position>>>,
    pub(crate) shared_triangle_buffers: Vec<Handle<TriangleBuffer<u32>>>,

    // Temporary buffers where we will write the mesh data
    pub(crate) temp_vertices: VertexBuffer<XYZW<f32>>,
    pub(crate) temp_triangles: TriangleBuffer<u32>,

    // These mesh shaders will take in the voxel data given from the voxel texture
    // and will use a compute shader that will utilize the surface nets algorithm
    // to generate an appropriate mesh for a chunk
    pub(crate) compute_vertices: ComputeShader,
    pub(crate) compute_voxels: ComputeShader,
    pub(crate) cached_indices: CachedIndices,
    
    // Guesser compute shader that will look into the memory chunks and 
    // try to find a free chunk of memory that we can use
    pub(crate) compute_find: ComputeShader,

    // Length = 4
    pub(crate) offsets: Buffer<u32>,
    
    // Length = 3
    pub(crate) counters: Buffer<u32>,

    // Used sub-memory chunks that contains a specific size of bytes per allocations
    pub(crate) sub_allocation_chunk_indices: Vec<Buffer<u32>>,
    
    // Copy shader that will copy the temporary vertices and triangles to the
    // output vertices and shaders within each block 
    pub(crate) compute_copy: ComputeShader,

    // All compute shaders use the same local dispatch work group size
    pub(crate) dispatch: u32,

    // Terrain generator will also be responsible for chunks
    pub(crate) chunks: AHashSet<ChunkCoords>,
    pub(crate) entities: AHashMap<ChunkCoords, Entity>,
    pub(crate) size: u32,
    pub(crate) allocations: usize,
    pub(crate) sub_allocations: usize,
    pub(crate) chunks_per_allocation: usize,
    pub(crate) material: Handle<TerrainMaterial>,
    pub(crate) id: MaterialId<TerrainMaterial>,

    // Keep a pool of all meshes and indirect buffers
    pub(crate) indirect_meshes: Vec<Handle<IndirectMesh>>,
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
            allocations,
            smoothing,
        } = settings;

        // Calculate the number of elements required for each triangle/vertex buffer allocation
        let scale_down = 1;
        let output_vertex_buffer_length = graphics.device().limits().max_storage_buffer_binding_size / 4 / 4 / scale_down;
        let output_triangle_buffer_length = graphics.device().limits().max_storage_buffer_binding_size / 4 / 3 / scale_down;
        log::warn!("output vertex buffer length: {output_vertex_buffer_length}");
        log::warn!("output triangle buffer length: {output_triangle_buffer_length}");
        
        // Calculate the number of chunk meshes/indirect elements that must be created
        let mut chunks = (chunk_render_distance * 2 + 1).pow(3);
        
        // Do this so each allocation contains the same amount of chunks
        chunks = ((chunks as f32 / allocations as f32).ceil() * (allocations as f32)) as u32;

        // Get the size of each sub-allocation
        // Number of sub allocations stays constant, the only thing that is
        // changed is the number of vertices and triangles per sub allocation
        let sub_allocations = 1024;
        
        // Get number of sub-allocation chunks for two buffer types (vertices and triangles)
        let vertex_sub_allocations_length = (output_vertex_buffer_length as f32) / sub_allocations as f32;
        let triangle_sub_allocations_length = (output_triangle_buffer_length as f32) / sub_allocations as f32;
        let vertices_per_sub_allocation = (vertex_sub_allocations_length.floor() as u32).next_power_of_two();
        let triangles_per_sub_allocation = (triangle_sub_allocations_length.floor() as u32).next_power_of_two();
        log::warn!("vertex sub allocations length: {}", vertices_per_sub_allocation);
        log::warn!("triangle sub allocations length: {}", triangles_per_sub_allocation);

        // Load the required generation compute shaders
        let chunks_per_allocation = (chunks as usize) / allocations;
        let compute_voxels = load_compute_voxels_shaders(assets, graphics);
        let compute_vertices = load_compute_vertices_shader(assets, graphics, size, smoothing);
        let compute_quads = load_compute_quads_shader(assets, graphics, size);
        let compute_copy =  load_compute_copy_shader(assets, graphics,  output_triangle_buffer_length, output_vertex_buffer_length, allocations, chunks, size);
        let compute_find = load_compute_find_shader(assets, graphics, allocations, chunks_per_allocation, sub_allocations, vertices_per_sub_allocation, triangles_per_sub_allocation, size);

        // Create cached data used for generation
        let cached_indices = create_texture3d(graphics, size);
        let densities = create_texture3d(graphics, size);
        let temp_vertices = create_temp_vertices(graphics, size);
        let temp_triangles = create_temp_triangles(graphics, size);

        // Create a counter that will store the currently stored values
        let counters = create_counters(graphics, 2);

        // Create counters that will be used to find free mem allocation
        let offsets = create_counters(graphics, 2);

        // A buffer that will contain the ranges of free memory for each allocation
        // Multiple buffers (per allocation) that will contain the used chunk indices for each sub allocation
        let sub_allocation_chunk_indices = create_sub_allocation_chunk_indices(graphics, allocations, sub_allocations);

        // Create buffers for vertices
        let shared_vertex_buffers = create_vertex_buffers(
            graphics,
            vertices,
            allocations,
            output_vertex_buffer_length
        );

        // Create buffers for triangles
        let shared_triangle_buffers = create_triangle_buffers(
            graphics,
            triangles,
            allocations,
            output_triangle_buffer_length
        );

        // Create ONE buffer that will store the indirect arguments
        let indirect_buffers = create_draw_indexed_indirect_buffer(
            graphics,
            indirect_buffers,
            chunks,
        );

        // Pre-allocate the meshes
        let indirect_meshes = preallocate_meshes(
            shared_vertex_buffers.clone(),
            shared_triangle_buffers.clone(),
            indirect_meshes,
            vertices,
            indirect_buffers,
            chunks,
            size,
            chunk_render_distance,
            chunks_per_allocation,
        );

        // Calculate the dispatch size for mesh generation by assuming local size is 4
        let dispatch = (size) / 4;

        Self {
            compute_vertices,
            compute_quads,
            cached_indices,
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
            
            shared_vertex_buffers,
            shared_triangle_buffers,

            temp_vertices,
            temp_triangles,
            compute_copy,
            offsets,
            counters,
            compute_find,
            allocations,
            chunks_per_allocation,
            sub_allocation_chunk_indices,
            sub_allocations,
        }
    }
}

fn create_sub_allocation_chunk_indices(
    graphics: &Graphics,
    allocations: usize,
    sub_allocations: usize
) -> Vec<Buffer<u32>> {
    (0..allocations).into_iter().map(|_| {
        Buffer::<u32>::splatted(
            graphics,
            sub_allocations,
            u32::MAX,
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap()
    }).collect::<Vec<_>>()
}

// Create multiple buffers that contain the free ranges of each allocation
fn create_ranges(
    graphics: &Graphics,
    allocations: usize,
    chunks_count: u32
) -> Vec<Buffer<vek::Vec4<u32>>> {
    let ranges_per_allocations = chunks_count as usize / allocations;

    (0..allocations).into_iter().map(|_| {
        Buffer::<vek::Vec4<u32>>::zeroed(
            graphics,
            ranges_per_allocations,
            BufferMode::Dynamic,
            BufferUsage::STORAGE
        ).unwrap()
    }).collect()
}


// Creates multiple big triangle buffers that will contain our data
fn create_triangle_buffers(
    graphics: &Graphics,
    triangles: &mut Storage<TriangleBuffer<u32>>,
    allocations: usize,
    output_triangle_buffer_length: u32,
) -> Vec<Handle<TriangleBuffer<u32>>> {
    (0..allocations).into_iter().map(|_| {
        triangles.insert(TriangleBuffer::zeroed(
            graphics,
            output_triangle_buffer_length as usize,
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE
        ).unwrap())
    }).collect::<Vec<_>>()
}

// Creates multiple big vertex buffers that will contain our data
fn create_vertex_buffers(
    graphics: &Graphics,
    vertices: &mut Storage<AttributeBuffer<attributes::Position>>,
    allocations: usize,
    output_vertex_buffer_length: u32,
) -> Vec<Handle<AttributeBuffer<attributes::Position>>> {
    (0..allocations).into_iter().map(|_| {
        let value = AttributeBuffer::<attributes::Position>::zeroed(
            graphics,
            output_vertex_buffer_length as usize,
            BufferMode::Dynamic,
            BufferUsage::STORAGE | BufferUsage::WRITE
        ).unwrap();
        dbg!(value.raw());
        vertices.insert(value)
    }).collect::<Vec<_>>()
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

// Create a buffer that will contain all DrawIndexedIndirect elements
fn create_draw_indexed_indirect_buffer(
    graphics: &Graphics,
    buffers: &mut Storage<DrawIndexedIndirectBuffer>,
    chunks_count: u32,
) -> Handle<DrawIndexedIndirectBuffer> {
    let elements = vec![DrawIndexedIndirect {
        vertex_count: 0,
        instance_count: 1,
        base_index: 0,
        vertex_offset: 0,
        base_instance: 0,
    }; chunks_count as usize];

    buffers.insert(DrawIndexedIndirectBuffer::from_slice(
        graphics,
        &elements,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE
        ).unwrap()
    )
}

// Create the meshes that we will use for terrain generation before hand
fn preallocate_meshes(
    shared_vertex_buffers: Vec<Handle<AttributeBuffer<attributes::Position>>>,
    shared_triangle_buffers: Vec<Handle<TriangleBuffer<u32>>>,
    meshes: &mut Storage<IndirectMesh>,
    vertices: &mut Storage<AttributeBuffer<attributes::Position>>,
    indexed_indirect_buffer: Handle<DrawIndexedIndirectBuffer>,
    chunks_count: u32,
    chunk_size: u32,
    chunk_render_distance: u32,
    chunks_per_allocation: usize,
) -> Vec<Handle<IndirectMesh>> {
    (0..(chunks_count as usize)).into_iter().map(|i| {
        // Get the allocation index for this chunk
        let allocation = ((i as f32) / (chunks_per_allocation as f32)).floor() as usize;

        // Get the vertex and triangle buffers that will be shared for this group
        let vertex_buffer = &shared_vertex_buffers[allocation];
        let triangle_buffer = &shared_triangle_buffers[allocation];
        // Create the indirect mesh
        let mut mesh = IndirectMesh::from_handles(
            Some(vertex_buffer.clone()),
            None,
            None,
            None,
            triangle_buffer.clone(),
            indexed_indirect_buffer.clone(),
            i
        );

        // Set the bounding box of the mesh before hand
        mesh.set_aabb(Some(math::Aabb {
            min: vek::Vec3::zero(),
            max: vek::Vec3::one() * chunk_size as f32,
        }));

        // Insert the mesh into the storage
        let handle = meshes.insert(mesh);
        handle
    }).collect()
}

// Create counters that will help us generate the vertices
fn create_counters(graphics: &Graphics, count: usize) -> Buffer<u32> {
    Buffer::zeroed(
        graphics,
        count,
        BufferMode::Dynamic,
        BufferUsage::STORAGE | BufferUsage::WRITE,
    ).unwrap()
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
    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <u32 as GpuPod>::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );
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
            <vek::Vec4<f32> as GpuPod>::size() + u32::size() * 2,
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );

    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}

// Load the compute shader that will find a free memory range
fn load_compute_find_shader(
    assets: &Assets,
    graphics: &Graphics,
    allocations: usize,
    chunks_per_allocation: usize,
    sub_allocations: usize,
    vertices_per_sub_allocation: u32,
    triangles_per_sub_allocation: u32,
    size: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/find.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_buffer::<u32>("counters", true, true);
    compiler.use_storage_buffer::<u32>("offsets", true, false);

    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <u32 as GpuPod>::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );
    
    compiler
        .use_snippet("sub_allocations", format!("const uint sub_allocations = {};", sub_allocations));
    compiler
        .use_snippet("vertices_per_sub_allocation", format!("const uint vertices_per_sub_allocation = {};", vertices_per_sub_allocation));
    compiler
        .use_snippet("triangles_per_sub_allocation", format!("const uint triangles_per_sub_allocation = {};", triangles_per_sub_allocation));
    
    compiler.use_storage_buffer::<u32>("indices", true, true);
    
    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}

// Load the compute shader that will copy the temp data to perm allocation space
fn load_compute_copy_shader(
    assets: &Assets,
    graphics: &Graphics,
    output_triangle_buffer_length: u32,
    output_vertex_buffer_length: u32,
    allocations: usize,
    chunks: u32,
    size: u32,
) -> ComputeShader {
    let module = assets
        .load::<ComputeModule>("engine/shaders/terrain/copy.comp")
        .unwrap();

    // Create a simple compute shader compiler
    let mut compiler = Compiler::new(assets, graphics);
    compiler.use_storage_buffer::<u32>("counters", true, false);
    compiler.use_storage_buffer::<u32>("offsets", true, false);

    compiler.use_push_constant_layout(
        PushConstantLayout::single(
            <u32 as GpuPod>::size(),
            ModuleVisibility::Compute,
        )
        .unwrap(),
    );
    
    compiler
        .use_snippet("size", format!("const uint size = {size};"));
    compiler
        .use_snippet("output_triangles_count", format!("const uint output_triangles_count = {output_triangle_buffer_length};"));
    compiler
        .use_snippet("output_vertices_count", format!("const uint output_vertices_count = {output_vertex_buffer_length};"));
    compiler
        .use_snippet("allocation_count", format!("const uint allocation_count = {allocations};"));
    compiler
        .use_snippet("max_chunk_count", format!("const uint max_chunk_count = {chunks};"));
    
    compiler.use_storage_buffer::<DrawIndexedIndirect>("indirect", false, true);
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>("temporary_vertices", true, false);
    compiler.use_storage_buffer::<u32>("temporary_triangles", true, false);
    compiler.use_storage_buffer::<<XYZW<f32> as Vertex>::Storage>("output_vertices", false, true);
    compiler.use_storage_buffer::<u32>("output_triangles", false, true);
    
    // Compile the compute shader
    let shader = ComputeShader::new(module, compiler).unwrap();
    shader
}