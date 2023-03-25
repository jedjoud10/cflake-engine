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
    float density = position.y - 20;
    position *= 0.06f;
    density += fbm(position * 0.1 * vec3(1, 2, 1), 6, 0.4f, 2.1f) * 35.25f;
    return density * 1010.0;
}