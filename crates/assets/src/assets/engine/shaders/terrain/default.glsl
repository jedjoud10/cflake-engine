// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    float time;
    vec3 _padding;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/cellular3D.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz;
    //float density = position.y - 20;
    //position *= 0.06f;
    float density = ((1 - cellular(position * 0.02).x) * 2 - 1.0) * 8.0 + position.y;
    return density;
}