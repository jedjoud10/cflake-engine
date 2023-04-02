// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    uint global_chunk_index;
    uint local_allocation_index;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz - vec3(0, 1, 0);
    //float density = length(position) - 20;
    float density = position.y;
    //density += ;
    float first = (1-fbmCellular(position * 0.004 * vec3(1, 1, 1), 9, 0.6, 1.8).x) * 32.0 - 50;
    float second = -erosion(position.xz * 0.04, 0.1).x * 140;
    density += mix(first, second, clamp((snoise(position * 0.001)) * 0.5 + 0.5, 0, 1));
    return density;
}

// TODO: Implement voxel struct and color function