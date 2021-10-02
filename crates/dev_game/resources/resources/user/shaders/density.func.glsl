#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
#include "user\shaders\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y;
    float c = (1-cellular(pos * vec3(1, 0.1, 1) * 0.001).x) * 6.0;
    float p = snoise(pos * vec3(1, 1, 1) * 0.002) * 10.0;
    density += snoise(pos * vec3(1, 1, 1) * 0.002) * 50;
    //density = pos.y - 10;
    return density;
}