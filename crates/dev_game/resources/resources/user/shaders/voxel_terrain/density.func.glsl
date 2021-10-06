#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Actual function for voxels
    // FBM Cellular noise with 11 octaves
    float fd = 0;
    for(int i = 0; i < 11; i++) {
        fd -= (1-abs(snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)))) * 100 * pow(0.43, i);
    }

    // Add the FBM cellular noise
    float density = pos.y + fd;

    // Make the terrain flatter
    density = min(density + 80, pos.y - 16.0);

    // Subtract a box from the terrain
    density = max(-sdSphere(pos, 100.0), density);
    return density;
}