#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Actual function for voxels
    // FBM Invertex billow noise with 8 octaves
    float fd = 0;
    for(int i = 0; i < 8; i++) {
        fd -= (1-abs(snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)))) * 100 * pow(0.43, i);
    }

    // Add the noise
    float density = pos.y + fd;

    // Subtract a sphere from the terrain
    density = max(-sdSphere(pos, 10.0), density);

    // Make the terrain flatter
    density = min(density + 80, pos.y - 16.0);
    return density;
}