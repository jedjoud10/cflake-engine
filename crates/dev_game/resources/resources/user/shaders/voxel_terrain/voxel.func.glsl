#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
// Generate the voxel data here
VoxelData get_voxel(vec3 pos) {
    // Actual function for voxels
    // Actual function for voxels
    // FBM Invertex billow noise with 8 octaves
    float fd = 0;
    for(int i = 0; i < 8; i++) {
        fd -= (1-abs(snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)))) * 100 * pow(0.43, i);
    }

    // Add the noise
    float density = pos.y + fd;

    // Make the terrain flatter
    density = opSmoothUnion(density + 80, pos.y - 16.0, 30.0);
    int material_id = 0;
    if (pos.x > 0) {
        material_id = 1;
    }
    return VoxelData(density * 20.0, 0, material_id);
}