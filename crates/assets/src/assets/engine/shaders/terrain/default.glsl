// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    uint global_chunk_index;
    uint local_allocation_index;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz - vec3(0, 1, 0);
    float density = position.y;
    density += fbm(position * 0.004 * vec3(1, 3, 1) + snoise(position * 0.01) * 0.1, 20, 0.5, 1.5) * 32.0 - 50;
    return density;
}

// TODO: Implement voxel struct and color function