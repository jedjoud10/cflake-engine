#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Generate the voxel data here
VoxelData get_voxel(vec3 pos) {
    // Actual function for voxels
    // FBM Invertex billow noise with 8 octaves
    float fd = 0;
    for(int i = 0; i < 8; i++) {
        fd -= (1-abs(snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)))) * 100 * pow(0.43, i);
    }

    // Add the noise
    float density = pos.y + fd;

    // Make the terrain flatter
    density = min(density + 80, pos.y - 16.0);
    return density;
}
// Generate the Vertex Color, Smoothness, Metallic and Material ID
EffectsVoxelData get_effects_voxel(vec3 pos) {
    return EffectVoxelData
}