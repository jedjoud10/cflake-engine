// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    //vec3 offset;
    float time;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    //position += parameters.offset;
    return snoise(position * 0.02 + vec3(parameters.time, 0.0, parameters.time) * 0.1) * 45 + position.y - 20.0;
}