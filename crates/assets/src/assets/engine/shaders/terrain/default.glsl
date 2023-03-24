// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    float time;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    return snoise(position * 0.02);
}