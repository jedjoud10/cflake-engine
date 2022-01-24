use main::{
    ecs::{entity::EntityID, impl_component},
    math::{
        self,
        octrees::{AdvancedOctree, Octree, OctreeNode},
    },
    rendering::{
        advanced::compute::ComputeShader,
        basics::{
            shader::ShaderSettings,
            texture::{Texture, TextureFilter, TextureFormat, TextureType, TextureWrapping},
        },
        object::ObjectID,
        pipeline::pipec,
        utils::DataType,
    },
    terrain::{ChunkCoords, MAIN_CHUNK_SIZE},
};
use std::collections::HashMap;

// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Chunk generation
    pub octree: AdvancedOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub csgtree: math::csg::CSGTree,

    // Voxel Generation
    pub compute_shader: ObjectID<ComputeShader>,
    pub voxel_texture: ObjectID<Texture>,
    pub material_texture: ObjectID<Texture>,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(octree_depth: u8, mut interpreter: main::terrain::interpreter::Interpreter, pipeline: &main::rendering::pipeline::Pipeline) -> Self {
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
        let (string, csgtree) = interpreter.finalize().unwrap();
        let ss = ShaderSettings::default().external_code(0, string).source(main::terrain::DEFAULT_TERRAIN_COMPUTE_SHADER);
        let compute_shader = ComputeShader::new(ss).unwrap();
        let compute_shader = pipec::construct(compute_shader, pipeline);

        // Create le textures
        // Create the voxel texture
        let voxel_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_format(TextureFormat::R32F)
            .set_data_type(DataType::F32)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);
        let material_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_format(TextureFormat::RG8I)
            .set_data_type(DataType::U8)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::ClampToBorder);

        // Now we actually need to construct the texture
        let voxel_texture = pipec::construct(voxel_texture, pipeline);
        let material_texture = pipec::construct(material_texture, pipeline);

        Self {
            octree,
            chunks: HashMap::default(),
            csgtree,

            compute_shader,
            voxel_texture,
            material_texture,
        }
    }
}

impl_component!(Terrain);
