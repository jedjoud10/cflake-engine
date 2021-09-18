#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
#include "user\shaders\erosion.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    // Do some position flipping
    float density = pos.y + 800;
    density -= mountain(pos.xz * 0.001, 1.0).x * 1600;
    float sphere = max(sdSphere(pos, 100), -sdSphere(pos, 50));
    density = max(density, -sphere);
    //density = pos.y - 10;
    return density;
}