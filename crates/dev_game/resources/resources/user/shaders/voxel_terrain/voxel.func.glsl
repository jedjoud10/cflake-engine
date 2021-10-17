#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the voxel data here
VoxelData get_voxel(vec3 pos) {
    // Actual function for voxels
    float density = pos.y / 20.0;
    return VoxelData(density, 0, 0);
}