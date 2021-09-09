#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y - 10.1;
    //density -= (1 - abs(snoise(pos * 0.001))) * 400.0;
    //density = opSubtraction(density, sdBox(pos, vec3(100.1, 100.1, 100.1)));
    density = pos.y - 10.1;
    return density;
}