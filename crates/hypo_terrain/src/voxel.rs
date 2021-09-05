use super::terrain::Terrain;
use super::CHUNK_SIZE;

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (CHUNK_SIZE);
    index -= z * (CHUNK_SIZE);
    let y = index / (CHUNK_SIZE * CHUNK_SIZE);
    let x = index % (CHUNK_SIZE);
    return (x, y, z);
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    return position.0 + (position.1 * CHUNK_SIZE * CHUNK_SIZE) + (position.2 * CHUNK_SIZE);
}

// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {}

impl VoxelGenerator {
    // Generate the voxels
    pub fn generate_voxels(&self, size: u64, position: veclib::Vector3<i64>, data: &mut Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>) -> Option<()> {
        let mut i = 0;
        let mut min: f32 = f32::MAX;
        let mut max: f32 = f32::MIN;
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    // Get the point in world coordinates
                    let size = size as f32 / (CHUNK_SIZE as f32 - 2.0);
                    let point: veclib::Vector3<f32> = veclib::Vector3::<f32>::new(x as f32, y as f32, z as f32) * size + veclib::Vector3::<f32>::from(position);
                    // Set the voxel data
                    data[i] = self.get_voxel(point);
                    // Keep track of the min max values
                    min = min.min(data[i].density);
                    max = max.max(data[i].density);
                    i += 1;
                }
            }
        }
        // Check if this data has a surface
        if min.signum() != max.signum() {
            // We have a surface
            return Some(());
        } else {
            // We do not have a surface
            return None;
        }
    }
}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
    pub color: veclib::Vector3<f32>,
}

impl VoxelGenerator {
    // Set the default values
    pub fn set_values(&mut self, terrain: &Terrain) {}
    // Get the voxel at a specific point
    pub fn get_voxel(&self, point: veclib::Vector3<f32>) -> Voxel {
        let mut voxel: Voxel = Voxel {
            density: 0.0,
            color: veclib::Vector3::default_zero(),
        };
        // Code goes here
        voxel.density = point.y() - 40.0;
        voxel.density += (point.x() * 0.05).sin() * 10.0;
        //voxel.density += (point.y() * 0.05).sin() * 3.0;
        voxel.density = (point.x() * 0.02).sin() * 20.0 + (point.y() * 0.02).sin() * 60.0 + (point.z() * 0.04).sin() * 60.0 + point.y() * 2.0;
        //voxel.density = voxel.density.max(point.y() - 40.0);
        //voxel.density = (point.x() - 5.0).min(point.y() - 5.0).min(point.z() - 5.0);
        //voxel.density = (-point.z() + 5.0).min(-point.x() + 5.0);
        //voxel.density = point.z() + point.y() + point.x() - 10.0;
        voxel.density -= 0.5;
        //voxel.density = point.y() - 5.5;
        //voxel.density = point.y() - 14.5 + (point.x() * 0.123).sin() * 30.0;
        // BIG NOTE: If the density value has no decimal, the skirts won't show up!
        return voxel;
    }
}
