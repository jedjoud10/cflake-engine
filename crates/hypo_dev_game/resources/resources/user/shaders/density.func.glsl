#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y - 10.1;
    //density = pos.y - 10.1;
    return density;
}