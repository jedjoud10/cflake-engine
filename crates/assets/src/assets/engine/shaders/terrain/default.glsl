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
    return snoise(position * 0.02) * 10.0 + position.y;

    /*
    float density = position.y - 20;
    position *= 0.06f;
    density += (fbm(position * 40.03, 1, 0.5f, 2.0f) * 0.25f) + (fbm(position * 1.96, 2, 0.5f, 2.0f) * 0.50f) + (fbm(position * 1.01, 3, 0.5f, 2.0f) * 1.00f);
    return density * 0.06f;
    */
}