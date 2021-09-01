use super::terrain::Terrain;
// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
}

impl VoxelGenerator {
    // Set the default values
    pub fn set_values(&mut self, terrain: &Terrain) {}
    // Get the voxel at a specific point
    pub fn get_voxel(&self, point: veclib::Vector3<f32>) -> Voxel {
        let mut voxel: Voxel = Voxel { density: 0.0 };
        // Code goes here
        voxel.density = point.y() - 40.0;
        voxel.density += (point.x() * 0.05).sin() * 10.0;
        voxel.density += (point.z() * 0.05).sin() * 3.0;
        //voxel.density = point.y() - 5.0;
        return voxel;
    }
}
