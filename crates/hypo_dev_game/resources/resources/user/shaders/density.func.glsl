#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
#include "user\shaders\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y - 10.1;
    float t = snoise(pos * 0.002) * 0.7;
    density += snoise(pos * 0.005 * vec3(1, 0.2, 1) + vec3(t, t, t)).x * 100.0;
    return density * 0.5;
}