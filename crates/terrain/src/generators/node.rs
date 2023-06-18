// A voxel node that is part of the voxel tree
pub trait VoxelNode {
    // Min/max range of the values this node might create
    fn range() -> (Option<f32>, Option<f32>);
}