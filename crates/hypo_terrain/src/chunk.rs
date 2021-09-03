use super::{
    terrain::{Terrain, CHUNK_SIZE},
    voxel::{Voxel, VoxelGenerator},
};
use hypo_ecs::*;
use hypo_rendering::{Model, ProceduralModelGenerator};

// Tables
use super::tables::*;

// A component that will be added to well... chunks
pub struct Chunk {
    pub position: veclib::Vector3<i64>,
    pub size: u64,
    pub data: Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            position: veclib::Vector3::<i64>::default_zero(),
            size: 0,
            data: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

// Main traits implemented
impl ComponentInternal for Chunk {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl ComponentID for Chunk {
    fn get_component_name() -> String {
        String::from("Chunk")
    }
}
impl Component for Chunk {}

impl Chunk {    
    // Generate the voxel data needed for mesh construction
    pub fn generate_data(&mut self, voxel_generator: &VoxelGenerator) -> (f32, f32) {
        let mut i = 0;
        let mut min: f32 = f32::MAX;
        let mut max: f32 = f32::MIN;
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Get the point in world coordinates
                    let size = self.size as f32 / (CHUNK_SIZE as f32 - 2.0);
                    let point: veclib::Vector3<f32> = veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32) * size + veclib::Vector3::<f32>::from(self.position);
                    // Set the voxel data
                    self.data[i] = voxel_generator.get_voxel(point);
                    // Keep track of the min max values
                    min = min.min(self.data[i].density);
                    max = max.max(self.data[i].density);
                    i += 1;
                }
            }
        }
        return (min, max);
    }
}

// This is a procedural model generator
impl ProceduralModelGenerator for Chunk {
    // Generate a procedural marching cube model
    fn generate_model(&self) -> Model {
        super::mesher::generate_model(&self.data)
    }
}
