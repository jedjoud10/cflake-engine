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
    pub compute_shader: ObjectID<ComputeShader>,
    // Textures
    pub density_texture: ObjectID<Texture>,
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
            .source(main::terrain::DEFAULT_TERRAIN_COMPUTE_SHADER);
        let compute_shader = ComputeShader::new(ss).unwrap();
        let compute_shader = pipec::construct(compute_shader, pipeline);

        // Create le textures
        // Create the voxel texture
        let voxel_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 1) as u16,
                (MAIN_CHUNK_SIZE + 1) as u16,
                (MAIN_CHUNK_SIZE + 1) as u16,
            ))
            .set_format(TextureFormat::R32F)
            .set_data_type(DataType::F32)
            .set_filter(TextureFilter::Nearest)
            .set_mipmaps(false)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);
        let material_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 1) as u16,
                (MAIN_CHUNK_SIZE + 1) as u16,
                (MAIN_CHUNK_SIZE + 1) as u16,
            ))
            .set_format(TextureFormat::RG8I)
            .set_data_type(DataType::U8)
            .set_filter(TextureFilter::Nearest)
            .set_mipmaps(false)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);

        // Now we actually need to construct the textures
        let voxel_texture = pipec::construct(voxel_texture, pipeline);
        let material_texture = pipec::construct(material_texture, pipeline);

        // Also construct the atomic
        let atomic = pipec::construct(AtomicGroup::new(&[0, 0]).unwrap().set_clear_condition(ClearCondition::BeforeShaderExecution), pipeline);

        
        Self {
            octree,
            chunks: HashMap::default(),
            material,

            generating: None,
            compute_shader,
            density_texture: voxel_texture,
            material_texture,
            counters: atomic
        }
    }
}

impl_component!(Terrain);
