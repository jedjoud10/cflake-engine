// Voxel states stored in multiple integers, using bitshifting and bit magic
pub struct VoxelState(u8);

const BITS_PER_STATE: usize = 2;
const STATES_PER_ROW: usize = u32::BITS as usize / BITS_PER_STATE;

// Set that contains the multiple voxel states
pub struct VoxelStateSet {
    vec: Vec<u32>,
}
