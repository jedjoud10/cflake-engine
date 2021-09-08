#include "user\shaders\noise.func.glsl"
#include "user\shaders\sdf.func.glsl"
// Generate the density here
float get_density(vec3 pos) {
    float density = pos.y - 10.1;
    density = (sdBox(pos, vec3(10.1, 10.1, 10.1)));
    density = pos.x - 10.1;
    return density;
}