#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the voxel data here
VoxelData get_voxel(vec3 pos) {
    // Actual function for voxels
    
    return VoxelData(int(pos.y), 0, 0);
}