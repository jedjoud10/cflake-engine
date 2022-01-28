use main::{
    ecs::{entity::EntityID, impl_component, component::ComponentID},
    math::{
        self,
        octrees::{AdvancedOctree, Octree, OctreeNode},
    },
    rendering::{
        advanced::{compute::ComputeShader, atomic::{AtomicGroup, AtomicGroupRead, ClearCondition}},
        basics::{
            shader::ShaderSettings,
            texture::{Texture, TextureFilter, TextureFormat, TextureType, TextureWrapping, TextureReadBytes}, material::Material,
        },
        object::{ObjectID, TrackedTaskID},
        pipeline::pipec,
        utils::DataType,
    },
    terrain::{ChunkCoords, VoxelData, MAIN_CHUNK_SIZE},
};
use std::collections::HashMap;

// Some data that we store whenever we are generating the voxels
pub struct TerrainGenerationData {
    // The ID of the main tracked task 
    pub main_id: TrackedTaskID,
    // The Entity ID of the chunk that we are generating this voxel data for
    pub chunk_id: EntityID, 

    // Reading the data back
    pub texture_reads: (TextureReadBytes, TextureReadBytes),
    pub atomic_read: AtomicGroupRead
}

// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: AdvancedOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub material: ObjectID<Material>,

    // Voxel Generation
    pub generating: Option<TerrainGenerationData>,
    pub base_compute: ObjectID<ComputeShader>,
    pub second_compute: ObjectID<ComputeShader>,
    // Textures
    pub base_texture: ObjectID<Texture>,
    pub normals_texture: ObjectID<Texture>,
    pub material_texture: ObjectID<Texture>,
    // Atomics
    pub counters: ObjectID<AtomicGroup>,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(material: ObjectID<Material>, octree_depth: u8, pipeline: &main::rendering::pipeline::Pipeline) -> Self {
        // Check if a an already existing node could be subdivided even more
        fn can_node_subdivide_twin(node: &OctreeNode, target: &veclib::Vector3<f32>, lod_factor: f32, max_depth: u8) -> bool {
            let c: veclib::Vector3<f32> = node.get_center().into();
            let max = node.depth == 1 || node.depth == 2;
            let result = c.distance(*target) < (node.half_extent as f32 * lod_factor) || max;
            node.children_indices.is_none() && node.depth < max_depth && result
        }
        // Create a new octree
        let internal_octree = Octree::new(octree_depth, (MAIN_CHUNK_SIZE) as u64);
        let octree = AdvancedOctree::new(internal_octree, can_node_subdivide_twin);

        // Load the compute shader
        let ss = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_BASE_COMPUTE_SHADER);
        let base_compute = ComputeShader::new(ss).unwrap();
        let base_compute = pipec::construct(base_compute, pipeline);

        let ss = ShaderSettings::default()
            .source(main::terrain::DEFAULT_TERRAIN_SECOND_COMPUTE_SHADER);
        let second_compute = ComputeShader::new(ss).unwrap();
        let second_compute = pipec::construct(second_compute, pipeline);

        // Create le textures
        let texture_dimensions = TextureType::Texture3D(
            (MAIN_CHUNK_SIZE + 2) as u16,
            (MAIN_CHUNK_SIZE + 2) as u16,
            (MAIN_CHUNK_SIZE + 2) as u16,
        );
        let texture_dimension_minus_one = TextureType::Texture3D(
            (MAIN_CHUNK_SIZE + 1) as u16,
            (MAIN_CHUNK_SIZE + 1) as u16,
            (MAIN_CHUNK_SIZE + 1) as u16,
        );
        // Create the textures
        let base_texture = Texture::default()
            .set_dimensions(texture_dimensions)
            .set_format(TextureFormat::RGBA32F)
            .set_data_type(DataType::F32)
            .set_filter(TextureFilter::Nearest)
            .set_mipmaps(false)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);
        let material_texture = Texture::default()
            .set_dimensions(texture_dimension_minus_one)
            .set_format(TextureFormat::RG8I)
            .set_data_type(DataType::U8)
            .set_filter(TextureFilter::Nearest)
            .set_mipmaps(false)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);
        let normals_texture = Texture::default()
            .set_dimensions(texture_dimension_minus_one)
            .set_format(TextureFormat::RGB16R)
            .set_data_type(DataType::I16)
            .set_filter(TextureFilter::Nearest)
            .set_mipmaps(false)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);

        // Now we actually need to construct the textures
        let base_texture = pipec::construct(base_texture, pipeline);
        let material_texture = pipec::construct(material_texture, pipeline);
        let normals_texture = pipec::construct(normals_texture, pipeline);

        // Also construct the atomic
        let atomic = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), pipeline);

        
        Self {
            octree,
            chunks: HashMap::default(),
            material,

            generating: None,
            compute_shader,
            base_texture,
            material_texture,
            normals_texture,
            counters: atomic
        }
    }
}

impl_component!(Terrain);
