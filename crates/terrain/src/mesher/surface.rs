// A generated marching cube
pub struct GeneratedCube {
    // The average vertex position
    pub average: vek::Vec3<f32>,

    // The cube's voxel material
    pub material: u8,
}

// A marching cube mesh's surface
#[derive(Default)]
pub struct GeneratedMeshSurface {
    // All the generated cubes and their average vertex position
    pub cubes: Vec<GeneratedCube>,
}
