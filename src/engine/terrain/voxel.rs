use super::Terrain;

// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {

}

// Just a simple voxel
#[derive(Default)]
pub struct Voxel {
    pub density: f32,
}

impl VoxelGenerator {
    // Set the default values
    pub fn set_values(&mut self, terrain: &Terrain) {

    }
    // Get the voxel at a specific point
    pub fn get_voxel(&self, point: glam::Vec3) -> Voxel {
        let mut voxel: Voxel = Voxel { density: 0.0 };
        // Code goes here
        voxel.density = point.y - 16.0;
        return voxel;
    }
}