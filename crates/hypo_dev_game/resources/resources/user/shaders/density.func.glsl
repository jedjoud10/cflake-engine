#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    pos = pos;
    float density = pos.y - 10.1;
    density += (snoise(vec3(pos.x * 0.001, 0.0, pos.z * 0.001))) * 300.0;
    //density = pos.y - 10.1;
    return density;
}