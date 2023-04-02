// Terrain voxel generation push constants
layout(push_constant) uniform PushConstants {
    vec4 offset;
    uint global_chunk_index;
    uint local_allocation_index;
} parameters;

// Load up some noise functions
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/noises/erosion2D.glsl>
#include <engine/shaders/sdf/common.glsl>
#include <engine/shaders/sdf/operations.glsl>
#include <engine/shaders/noises/fbm.glsl>

// Main density function that will create the shape of the terrain
float density(vec3 position) {
    position += parameters.offset.xyz - vec3(0, 1, 0);

    /*
    //float density = length(position) - 20;
    float density = position.y;
    //density += ;
    */

    float density = opSmoothUnion(-erosion(position.xz * 0.04, 0.1).x * 180 + position.y + 70, position.y, 20);
    
    return density;
}

// TODO: Implement voxel struct and color function