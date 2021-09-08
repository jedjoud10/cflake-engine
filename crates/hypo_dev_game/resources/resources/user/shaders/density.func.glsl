//#include "user\\shader\\noise.func.glsl"
// Generate the density here
float density(vec3 pos) {
    return pos.y - 5.1;
}