#include "defaults\shaders\voxel_terrain\noise.func.glsl"
#include "defaults\shaders\voxel_terrain\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y;
    float c = (1-cellular(pos * vec3(1, 0.1, 1) * 0.001).x) * 6.0;
    float p = snoise(pos * vec3(1, 1, 1) * 0.002) * 10.0;
    float fd = 0;
    for(int i = 0; i < 9; i++) {
        fd += (snoise(pos * vec3(1, 0.2, 1) * 0.001 * pow(2, i)).x) * 100 * pow(0.4, i);
    }
    density += clamp(fd, -50, 100);
    density = max(-sdBox(pos, vec3(50, 30, 50)), density);
    //density = pos.y - 10;
    return density;
}